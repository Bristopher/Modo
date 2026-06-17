// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(target_os = "linux")]
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[cfg(target_os = "linux")]
use futures_lite::{FutureExt, StreamExt, future};
#[cfg(target_os = "linux")]
use zbus::{
    MatchRule, MessageStream, Proxy,
    message::Type as MessageType,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

#[cfg(target_os = "linux")]
use crate::portal::{REQUEST_INTERFACE, mint_token, request_path};

pub const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
pub const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
pub const GLOBAL_SHORTCUTS_INTERFACE: &str = "org.freedesktop.portal.GlobalShortcuts";
pub const SESSION_INTERFACE: &str = "org.freedesktop.portal.Session";

#[cfg(target_os = "linux")]
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(5 * 60);
#[cfg(target_os = "linux")]
pub const SIGNAL_POLL_INTERVAL: Duration = Duration::from_millis(200);
#[cfg(target_os = "linux")]
pub const SIGNAL_THREAD_START_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(target_os = "linux")]
type ShortcutProperties = HashMap<String, OwnedValue>;
#[cfg(target_os = "linux")]
type ShortcutsChangedBody = (OwnedObjectPath, Vec<(String, ShortcutProperties)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutEntry {
    pub id: String,
    pub description: String,
    pub preferred_trigger: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundShortcut {
    pub id: String,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureResult {
    pub action: String,
    pub shortcuts: Vec<BoundShortcut>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutEvent {
    Activated { id: String },
    Deactivated { id: String },
    ShortcutsChanged { shortcuts: Vec<BoundShortcut> },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalShortcutsError {
    DbusError,
    PortalTimeout,
    InvalidReply,
    SendFailed,
    Cancelled,
    ThreadStartFailed,
    LockPoisoned,
}

impl std::fmt::Display for GlobalShortcutsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DbusError => "DbusError",
            Self::PortalTimeout => "PortalTimeout",
            Self::InvalidReply => "InvalidReply",
            Self::SendFailed => "SendFailed",
            Self::Cancelled => "Cancelled",
            Self::ThreadStartFailed => "ThreadStartFailed",
            Self::LockPoisoned => "LockPoisoned",
        };
        f.write_str(s)
    }
}

#[cfg(target_os = "linux")]
type ShortcutCallback = Arc<dyn Fn(ShortcutEvent) + Send + Sync + 'static>;

#[cfg(target_os = "linux")]
pub struct Subscription {
    stop_flag: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(target_os = "linux")]
impl Subscription {
    pub fn configure(
        entries: Vec<ShortcutEntry>,
        callback: ShortcutCallback,
    ) -> Result<(Self, ConfigureResult), GlobalShortcutsError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_for_thread = stop_flag.clone();
        let (ready_tx, ready_rx) =
            mpsc::sync_channel::<Result<ConfigureReady, GlobalShortcutsError>>(1);
        let thread = thread::Builder::new()
            .name("fluxer-linux-portals-global-shortcuts".to_string())
            .spawn(move || {
                let setup = future::block_on(async {
                    let conn = zbus::Connection::session()
                        .await
                        .map_err(|_| GlobalShortcutsError::DbusError)?;
                    let rule = MatchRule::builder().msg_type(MessageType::Signal).build();
                    let stream = MessageStream::for_match_rule(rule, &conn, Some(64))
                        .await
                        .map_err(|_| GlobalShortcutsError::DbusError)?;
                    let ready = configure_session(&conn, &entries).await?;
                    Ok::<_, GlobalShortcutsError>((conn, stream, ready))
                });
                let (conn, mut stream, ready) = match setup {
                    Ok(parts) => parts,
                    Err(err) => {
                        let _ = ready_tx.send(Err(err));
                        return;
                    }
                };
                let session_handle = ready.session_handle.clone();
                if ready_tx.send(Ok(ready)).is_err() {
                    let _ = future::block_on(close_session(&conn, &session_handle));
                    return;
                }
                while !stop_for_thread.load(Ordering::Acquire) {
                    let timeout = async {
                        async_io::Timer::after(SIGNAL_POLL_INTERVAL).await;
                        None::<zbus::Result<zbus::Message>>
                    };
                    match future::block_on(stream.next().or(timeout)) {
                        Some(Ok(message)) => {
                            if let Some(event) = parse_signal(&message, &session_handle) {
                                let closed = matches!(event, ShortcutEvent::Closed);
                                callback(event);
                                if closed {
                                    return;
                                }
                            }
                        }
                        Some(Err(_)) => break,
                        None => {}
                    }
                }
                let _ = future::block_on(close_session(&conn, &session_handle));
            })
            .map_err(|_| GlobalShortcutsError::ThreadStartFailed)?;

        match ready_rx.recv_timeout(SIGNAL_THREAD_START_TIMEOUT) {
            Ok(Ok(ready)) => Ok((
                Self {
                    stop_flag,
                    thread: Mutex::new(Some(thread)),
                },
                ready.result,
            )),
            Ok(Err(err)) => {
                let _ = thread.join();
                Err(err)
            }
            Err(err) => {
                stop_flag.store(true, Ordering::Release);
                if matches!(err, mpsc::RecvTimeoutError::Disconnected) {
                    let _ = thread.join();
                }
                Err(GlobalShortcutsError::ThreadStartFailed)
            }
        }
    }

    pub fn close(&self) {
        self.stop_flag.store(true, Ordering::Release);
        if let Ok(mut thread) = self.thread.lock()
            && let Some(t) = thread.take()
        {
            let _ = t.join();
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for Subscription {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(target_os = "linux")]
struct ConfigureReady {
    session_handle: String,
    result: ConfigureResult,
}

#[cfg(target_os = "linux")]
struct PortalResponse {
    code: u32,
    results: HashMap<String, OwnedValue>,
}

#[cfg(target_os = "linux")]
async fn configure_session(
    conn: &zbus::Connection,
    entries: &[ShortcutEntry],
) -> Result<ConfigureReady, GlobalShortcutsError> {
    let session_handle = create_session(conn).await?;
    let persisted = list_shortcuts(conn, &session_handle)
        .await
        .unwrap_or_default();
    if shortcut_ids_match(entries, &persisted) {
        return Ok(ConfigureReady {
            session_handle,
            result: ConfigureResult {
                action: "listed".to_string(),
                shortcuts: persisted,
            },
        });
    }
    let bound = bind_shortcuts(conn, &session_handle, entries).await?;
    Ok(ConfigureReady {
        session_handle,
        result: bound,
    })
}

#[cfg(target_os = "linux")]
fn shortcut_ids_match(entries: &[ShortcutEntry], persisted: &[BoundShortcut]) -> bool {
    if entries.is_empty() || persisted.is_empty() {
        return false;
    }
    let requested: HashSet<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
    let existing: HashSet<&str> = persisted
        .iter()
        .map(|shortcut| shortcut.id.as_str())
        .collect();
    requested == existing
}

#[cfg(target_os = "linux")]
async fn create_session(conn: &zbus::Connection) -> Result<String, GlobalShortcutsError> {
    let handle_token = mint_token("fluxer_gs_create");
    let session_handle_token = mint_token("fluxer_gs_session");
    let mut stream = request_stream(conn, &handle_token).await?;
    let proxy = global_shortcuts_proxy(conn).await?;
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("handle_token", Value::new(handle_token.as_str()));
    options.insert(
        "session_handle_token",
        Value::new(session_handle_token.as_str()),
    );
    let _reply_path: OwnedObjectPath = proxy
        .call("CreateSession", &(options,))
        .await
        .map_err(|_| GlobalShortcutsError::SendFailed)?;
    let response = wait_for_response(&mut stream).await?;
    if response.code != 0 {
        return Err(GlobalShortcutsError::Cancelled);
    }
    response
        .results
        .get("session_handle")
        .and_then(|value| string_or_object_path(crate::kwin::value_of_owned(value)))
        .ok_or(GlobalShortcutsError::InvalidReply)
}

#[cfg(target_os = "linux")]
async fn list_shortcuts(
    conn: &zbus::Connection,
    session_handle: &str,
) -> Result<Vec<BoundShortcut>, GlobalShortcutsError> {
    let session_path = owned_path(session_handle)?;
    let handle_token = mint_token("fluxer_gs_list");
    let mut stream = request_stream(conn, &handle_token).await?;
    let proxy = global_shortcuts_proxy(conn).await?;
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("handle_token", Value::new(handle_token.as_str()));
    let _reply_path: OwnedObjectPath = proxy
        .call("ListShortcuts", &(&session_path, options))
        .await
        .map_err(|_| GlobalShortcutsError::SendFailed)?;
    let response = wait_for_response(&mut stream).await?;
    if response.code != 0 {
        return Err(GlobalShortcutsError::Cancelled);
    }
    Ok(shortcuts_from_results(&response.results))
}

#[cfg(target_os = "linux")]
async fn bind_shortcuts(
    conn: &zbus::Connection,
    session_handle: &str,
    entries: &[ShortcutEntry],
) -> Result<ConfigureResult, GlobalShortcutsError> {
    let session_path = owned_path(session_handle)?;
    let handle_token = mint_token("fluxer_gs_bind");
    let mut stream = request_stream(conn, &handle_token).await?;
    let proxy = global_shortcuts_proxy(conn).await?;
    let shortcuts = serialize_shortcuts(entries);
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("handle_token", Value::new(handle_token.as_str()));
    let _reply_path: OwnedObjectPath = proxy
        .call("BindShortcuts", &(&session_path, shortcuts, "", options))
        .await
        .map_err(|_| GlobalShortcutsError::SendFailed)?;
    let response = wait_for_response(&mut stream).await?;
    if response.code != 0 {
        return Ok(ConfigureResult {
            action: "cancelled".to_string(),
            shortcuts: Vec::new(),
        });
    }
    Ok(ConfigureResult {
        action: "bound".to_string(),
        shortcuts: shortcuts_from_results(&response.results),
    })
}

#[cfg(target_os = "linux")]
async fn global_shortcuts_proxy(
    conn: &zbus::Connection,
) -> Result<Proxy<'_>, GlobalShortcutsError> {
    Proxy::new(
        conn,
        PORTAL_DESTINATION,
        PORTAL_PATH,
        GLOBAL_SHORTCUTS_INTERFACE,
    )
    .await
    .map_err(|_| GlobalShortcutsError::DbusError)
}

async fn request_stream(
    conn: &zbus::Connection,
    handle_token: &str,
) -> Result<MessageStream, GlobalShortcutsError> {
    let unique_owned = conn
        .unique_name()
        .ok_or(GlobalShortcutsError::DbusError)?
        .to_owned();
    let unique_name = unique_owned.as_str().to_string();
    let expected_path = request_path(&unique_name, handle_token);
    let rule = MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface(REQUEST_INTERFACE)
        .map_err(|_| GlobalShortcutsError::DbusError)?
        .member("Response")
        .map_err(|_| GlobalShortcutsError::DbusError)?
        .path(expected_path.clone())
        .map_err(|_| GlobalShortcutsError::DbusError)?
        .build();
    MessageStream::for_match_rule(rule, conn, Some(8))
        .await
        .map_err(|_| GlobalShortcutsError::DbusError)
}

async fn wait_for_response(
    stream: &mut MessageStream,
) -> Result<PortalResponse, GlobalShortcutsError> {
    loop {
        let timeout = async {
            async_io::Timer::after(REQUEST_TIMEOUT).await;
            None::<zbus::Result<zbus::Message>>
        };
        match stream.next().or(timeout).await {
            Some(Ok(message)) => {
                if let Some(parsed) = parse_request_response(&message) {
                    return Ok(parsed);
                }
            }
            Some(Err(_)) => return Err(GlobalShortcutsError::DbusError),
            None => return Err(GlobalShortcutsError::PortalTimeout),
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_request_response(message: &zbus::Message) -> Option<PortalResponse> {
    let body = message.body();
    let (code, results): (u32, HashMap<String, OwnedValue>) = body.deserialize().ok()?;
    Some(PortalResponse { code, results })
}

#[cfg(target_os = "linux")]
fn serialize_shortcuts(entries: &[ShortcutEntry]) -> Vec<(&str, HashMap<&str, Value<'_>>)> {
    entries
        .iter()
        .map(|entry| {
            let mut options: HashMap<&str, Value<'_>> = HashMap::new();
            options.insert("description", Value::new(entry.description.as_str()));
            if let Some(trigger) = entry.preferred_trigger.as_deref() {
                options.insert("preferred_trigger", Value::new(trigger));
            }
            (entry.id.as_str(), options)
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn shortcuts_from_results(results: &HashMap<String, OwnedValue>) -> Vec<BoundShortcut> {
    results
        .get("shortcuts")
        .and_then(|value| shortcuts_from_value(crate::kwin::value_of_owned(value)))
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn shortcuts_from_value(value: &Value<'_>) -> Option<Vec<BoundShortcut>> {
    let inner = unbox_value(value);
    let Value::Array(array) = inner else {
        return None;
    };
    let mut shortcuts = Vec::new();
    for value in array.inner() {
        if let Some(shortcut) = bound_shortcut_from_value(value) {
            shortcuts.push(shortcut);
        }
    }
    Some(shortcuts)
}

#[cfg(target_os = "linux")]
fn bound_shortcut_from_value(value: &Value<'_>) -> Option<BoundShortcut> {
    let Value::Structure(structure) = unbox_value(value) else {
        return None;
    };
    let fields = structure.fields();
    if fields.len() < 2 {
        return None;
    }
    let id = string_or_object_path(&fields[0])?;
    let dict = match unbox_value(&fields[1]) {
        Value::Dict(dict) => Some(dict),
        _ => None,
    };
    Some(BoundShortcut {
        id,
        description: dict.and_then(|d| dict_string(d, "description")),
        trigger_description: dict.and_then(|d| dict_string(d, "trigger_description")),
    })
}

#[cfg(target_os = "linux")]
fn dict_string(dict: &zbus::zvariant::Dict<'_, '_>, key: &str) -> Option<String> {
    dict.iter().find_map(|(k, v)| {
        if string_or_object_path(k).as_deref() == Some(key) {
            string_or_object_path(unbox_value(v))
        } else {
            None
        }
    })
}

#[cfg(target_os = "linux")]
fn parse_signal(message: &zbus::Message, session_handle: &str) -> Option<ShortcutEvent> {
    let header = message.header();
    let interface = header.interface()?.as_str();
    let member = header.member()?.as_str();
    match (interface, member) {
        (GLOBAL_SHORTCUTS_INTERFACE, "Activated") => {
            let (session, id, _timestamp, _options): (
                OwnedObjectPath,
                String,
                u64,
                HashMap<String, OwnedValue>,
            ) = message.body().deserialize().ok()?;
            if session.as_str() == session_handle {
                Some(ShortcutEvent::Activated { id })
            } else {
                None
            }
        }
        (GLOBAL_SHORTCUTS_INTERFACE, "Deactivated") => {
            let (session, id, _timestamp, _options): (
                OwnedObjectPath,
                String,
                u64,
                HashMap<String, OwnedValue>,
            ) = message.body().deserialize().ok()?;
            if session.as_str() == session_handle {
                Some(ShortcutEvent::Deactivated { id })
            } else {
                None
            }
        }
        (GLOBAL_SHORTCUTS_INTERFACE, "ShortcutsChanged") => {
            let (session, shortcuts): ShortcutsChangedBody = message.body().deserialize().ok()?;
            if session.as_str() != session_handle {
                return None;
            }
            Some(ShortcutEvent::ShortcutsChanged {
                shortcuts: shortcuts
                    .into_iter()
                    .map(|(id, properties)| bound_shortcut_from_parts(id, &properties))
                    .collect(),
            })
        }
        (SESSION_INTERFACE, "Closed") => {
            if header.path()?.as_str() == session_handle {
                Some(ShortcutEvent::Closed)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn bound_shortcut_from_parts(
    id: String,
    properties: &HashMap<String, OwnedValue>,
) -> BoundShortcut {
    BoundShortcut {
        id,
        description: properties
            .get("description")
            .and_then(|value| string_or_object_path(crate::kwin::value_of_owned(value))),
        trigger_description: properties
            .get("trigger_description")
            .and_then(|value| string_or_object_path(crate::kwin::value_of_owned(value))),
    }
}

#[cfg(target_os = "linux")]
async fn close_session(
    conn: &zbus::Connection,
    session_handle: &str,
) -> Result<(), GlobalShortcutsError> {
    let proxy = Proxy::new(conn, PORTAL_DESTINATION, session_handle, SESSION_INTERFACE)
        .await
        .map_err(|_| GlobalShortcutsError::DbusError)?;
    proxy
        .call::<_, _, ()>("Close", &())
        .await
        .map_err(|_| GlobalShortcutsError::SendFailed)
}

#[cfg(target_os = "linux")]
fn owned_path(path: &str) -> Result<OwnedObjectPath, GlobalShortcutsError> {
    OwnedObjectPath::try_from(path.to_string()).map_err(|_| GlobalShortcutsError::InvalidReply)
}

#[cfg(target_os = "linux")]
fn unbox_value<'a>(value: &'a Value<'a>) -> &'a Value<'a> {
    match value {
        Value::Value(inner) => unbox_value(inner),
        other => other,
    }
}

#[cfg(target_os = "linux")]
fn string_or_object_path(value: &Value<'_>) -> Option<String> {
    match unbox_value(value) {
        Value::Str(v) => Some(v.as_str().to_string()),
        Value::ObjectPath(v) => Some(v.as_str().to_string()),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
pub fn get_portal_version() -> Option<u32> {
    let conn = zbus::blocking::connection::Builder::session()
        .ok()?
        .method_timeout(Duration::from_millis(1_500))
        .build()
        .ok()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        PORTAL_DESTINATION,
        PORTAL_PATH,
        GLOBAL_SHORTCUTS_INTERFACE,
    )
    .ok()?;
    proxy.get_property("version").ok()
}

#[cfg(target_os = "linux")]
pub fn is_available() -> bool {
    get_portal_version().is_some()
}

#[cfg(not(target_os = "linux"))]
pub struct Subscription;

#[cfg(not(target_os = "linux"))]
impl Subscription {
    pub fn configure(
        _entries: Vec<ShortcutEntry>,
        _callback: Arc<dyn Fn(ShortcutEvent) + Send + Sync + 'static>,
    ) -> Result<(Self, ConfigureResult), GlobalShortcutsError> {
        Err(GlobalShortcutsError::DbusError)
    }

    pub fn close(&self) {}
}

#[cfg(not(target_os = "linux"))]
pub fn get_portal_version() -> Option<u32> {
    None
}

#[cfg(not(target_os = "linux"))]
pub fn is_available() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_id_match_requires_same_ids() {
        let entries = vec![ShortcutEntry {
            id: "one".into(),
            description: "One".into(),
            preferred_trigger: Some("CTRL+o".into()),
        }];
        let persisted = vec![BoundShortcut {
            id: "one".into(),
            description: None,
            trigger_description: None,
        }];
        assert!(shortcut_ids_match(&entries, &persisted));
    }

    #[test]
    fn shortcut_id_match_rejects_empty() {
        assert!(!shortcut_ids_match(&[], &[]));
    }
}
