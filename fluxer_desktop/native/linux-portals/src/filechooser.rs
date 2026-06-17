// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(target_os = "linux")]
use std::{collections::HashMap, sync::mpsc, time::Duration};

#[cfg(target_os = "linux")]
use futures_lite::{FutureExt, StreamExt, future};
#[cfg(target_os = "linux")]
use zbus::{
    MatchRule, MessageStream,
    blocking::{Connection as BlockingConnection, Proxy as BlockingProxy},
    message::Type as MessageType,
    zvariant::{OwnedObjectPath, Value},
};

#[cfg(target_os = "linux")]
use crate::portal::{REQUEST_INTERFACE, mint_token, request_path};

pub const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
pub const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
pub const FILE_CHOOSER_INTERFACE: &str = "org.freedesktop.portal.FileChooser";

#[cfg(target_os = "linux")]
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5 * 60);
#[cfg(target_os = "linux")]
pub const SIGNAL_POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Save,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    pub kind: u32,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub name: String,
    pub rules: Vec<FilterRule>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub parent_window: String,
    pub title: String,
    pub accept_label: Option<String>,
    pub modal: bool,
    pub multiple: bool,
    pub directory: bool,
    pub current_folder: Option<String>,
    pub current_name: Option<String>,
    pub current_file: Option<String>,
    pub filters: Vec<Filter>,
    pub current_filter: Option<Filter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChooserResult {
    pub cancelled: bool,
    pub uris: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChooserError {
    DbusError,
    PortalTimeout,
    InvalidReply,
    SendFailed,
}

impl std::fmt::Display for FileChooserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DbusError => "DbusError",
            Self::PortalTimeout => "PortalTimeout",
            Self::InvalidReply => "InvalidReply",
            Self::SendFailed => "SendFailed",
        };
        f.write_str(s)
    }
}

#[cfg(target_os = "linux")]
pub fn invoke(mode: Mode, options: Options) -> Result<FileChooserResult, FileChooserError> {
    let conn = zbus::blocking::connection::Builder::session()
        .map_err(|_| FileChooserError::DbusError)?
        .method_timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| FileChooserError::DbusError)?;
    let unique_owned = conn
        .unique_name()
        .ok_or(FileChooserError::DbusError)?
        .to_owned();
    let unique_name = unique_owned.as_str().to_string();

    let token_prefix = match mode {
        Mode::Open => "fluxer_fc_open",
        Mode::Save => "fluxer_fc_save",
    };
    let handle_token = mint_token(token_prefix);
    let expected_path = request_path(&unique_name, &handle_token);

    let (tx, rx) = mpsc::sync_channel::<FileChooserResponse>(1);
    let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_for_thread = stop_flag.clone();
    let expected_for_thread = expected_path.clone();
    let listener = std::thread::Builder::new()
        .name("fluxer-linux-portals-fc".to_string())
        .spawn(move || {
            response_listener(&expected_for_thread, tx, stop_for_thread);
        })
        .map_err(|_| FileChooserError::DbusError)?;

    let send_result = send_call(&conn, mode, &handle_token, &options, &expected_path);

    let result = match send_result {
        Ok(()) => match rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(response) => {
                if response.code != 0 {
                    Ok(FileChooserResult {
                        cancelled: true,
                        uris: vec![],
                    })
                } else {
                    Ok(FileChooserResult {
                        cancelled: false,
                        uris: response.uris,
                    })
                }
            }
            Err(_) => Err(FileChooserError::PortalTimeout),
        },
        Err(err) => Err(err),
    };

    stop_flag.store(true, std::sync::atomic::Ordering::Release);
    let _ = listener.join();
    result
}

#[cfg(target_os = "linux")]
struct FileChooserResponse {
    code: u32,
    uris: Vec<String>,
}

#[cfg(target_os = "linux")]
fn response_listener(
    expected_path: &str,
    tx: mpsc::SyncSender<FileChooserResponse>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let setup = future::block_on(async {
        let conn = zbus::Connection::session().await?;
        let rule = MatchRule::builder()
            .msg_type(MessageType::Signal)
            .interface(REQUEST_INTERFACE)?
            .member("Response")?
            .path(expected_path.to_string())?
            .build();
        let stream = MessageStream::for_match_rule(rule, &conn, Some(8)).await?;
        zbus::Result::Ok((conn, stream))
    });
    let (_conn, mut stream) = match setup {
        Ok(parts) => parts,
        Err(_) => return,
    };
    while !stop.load(std::sync::atomic::Ordering::Acquire) {
        let timeout = async {
            async_io::Timer::after(SIGNAL_POLL_INTERVAL).await;
            None::<zbus::Result<zbus::Message>>
        };
        match future::block_on(stream.next().or(timeout)) {
            Some(Ok(message)) => {
                if let Some(parsed) = parse_filechooser_response(&message) {
                    let _ = tx.send(parsed);
                    return;
                }
            }
            Some(Err(_)) => return,
            None => {}
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_filechooser_response(message: &zbus::Message) -> Option<FileChooserResponse> {
    let body = message.body();
    let (code, results): (u32, HashMap<String, zbus::zvariant::OwnedValue>) =
        body.deserialize().ok()?;
    let mut uris: Vec<String> = Vec::new();
    if let Some(v) = results.get("uris") {
        let val = crate::kwin::value_of_owned(v);
        if let Value::Array(arr) = val {
            for element in arr.iter() {
                let inner: &Value<'_> = match element {
                    Value::Value(b) => b.as_ref(),
                    other => other,
                };
                if let Value::Str(s) = inner {
                    uris.push(s.as_str().to_string());
                }
            }
        }
    }
    Some(FileChooserResponse { code, uris })
}

#[cfg(target_os = "linux")]
fn send_call(
    conn: &BlockingConnection,
    mode: Mode,
    handle_token: &str,
    options: &Options,
    expected_path: &str,
) -> Result<(), FileChooserError> {
    let proxy = BlockingProxy::new(
        conn,
        PORTAL_DESTINATION,
        PORTAL_PATH,
        FILE_CHOOSER_INTERFACE,
    )
    .map_err(|_| FileChooserError::DbusError)?;
    let member = match mode {
        Mode::Open => "OpenFile",
        Mode::Save => "SaveFile",
    };

    let mut vardict: HashMap<&str, Value<'_>> = HashMap::new();
    vardict.insert("handle_token", Value::new(handle_token));
    vardict.insert("modal", Value::new(options.modal));
    vardict.insert("multiple", Value::new(options.multiple));
    if matches!(mode, Mode::Open) && options.directory {
        vardict.insert("directory", Value::new(true));
    }
    if let Some(label) = options.accept_label.as_deref() {
        vardict.insert("accept_label", Value::new(label));
    }
    if !options.filters.is_empty() {
        vardict.insert("filters", Value::new(serialize_filters(&options.filters)));
    }
    if let Some(cf) = options.current_filter.as_ref() {
        vardict.insert("current_filter", Value::new(serialize_filter(cf)));
    }
    if let Some(folder) = options.current_folder.as_deref() {
        vardict.insert("current_folder", Value::new(folder.as_bytes()));
    }
    if matches!(mode, Mode::Save)
        && let Some(name) = options.current_name.as_deref()
    {
        vardict.insert("current_name", Value::new(name));
    }
    if matches!(mode, Mode::Save)
        && let Some(file) = options.current_file.as_deref()
    {
        vardict.insert("current_file", Value::new(file.as_bytes()));
    }

    let reply_path: OwnedObjectPath = proxy
        .call(
            member,
            &(
                options.parent_window.as_str(),
                options.title.as_str(),
                vardict,
            ),
        )
        .map_err(|_| FileChooserError::SendFailed)?;
    if !reply_path.as_str().is_empty() && reply_path.as_str() != expected_path {
        return Err(FileChooserError::InvalidReply);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn serialize_filters(filters: &[Filter]) -> Vec<(&str, Vec<(u32, &str)>)> {
    filters
        .iter()
        .map(|f| {
            let rules: Vec<(u32, &str)> = f
                .rules
                .iter()
                .map(|r| (r.kind, r.pattern.as_str()))
                .collect();
            (f.name.as_str(), rules)
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn serialize_filter(filter: &Filter) -> (&str, Vec<(u32, &str)>) {
    let rules: Vec<(u32, &str)> = filter
        .rules
        .iter()
        .map(|r| (r.kind, r.pattern.as_str()))
        .collect();
    (filter.name.as_str(), rules)
}

#[cfg(not(target_os = "linux"))]
pub fn invoke(_mode: Mode, _options: Options) -> Result<FileChooserResult, FileChooserError> {
    Err(FileChooserError::DbusError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_is_open_safe() {
        let opts = Options::default();
        assert!(!opts.directory);
        assert!(!opts.multiple);
        assert!(opts.parent_window.is_empty());
    }

    #[test]
    fn filter_rule_kinds_match_typescript_union() {
        let glob = FilterRule {
            kind: 0,
            pattern: "*.png".into(),
        };
        let mime = FilterRule {
            kind: 1,
            pattern: "image/png".into(),
        };
        assert_eq!(glob.kind, 0);
        assert_eq!(mime.kind, 1);
    }
}
