// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopSession {
    Kde,
    Gnome,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
    WaylandWithXwayland,
    Unknown,
}

pub fn has_dbus_session() -> bool {
    has_dbus_session_from(
        std::env::var("DBUS_SESSION_BUS_ADDRESS").ok().as_deref(),
        std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        |path| std::path::Path::new(path).exists(),
    )
}

fn has_dbus_session_from(
    bus_address: Option<&str>,
    xdg_runtime_dir: Option<&str>,
    path_exists: impl Fn(&str) -> bool,
) -> bool {
    if bus_address.is_some_and(|v| !v.is_empty()) {
        return true;
    }
    if let Some(dir) = xdg_runtime_dir
        && !dir.is_empty()
    {
        let candidate = format!("{}/bus", dir.trim_end_matches('/'));
        if path_exists(&candidate) {
            return true;
        }
    }
    false
}

impl DisplayServer {
    pub fn x11_reachable(self) -> bool {
        matches!(self, Self::X11 | Self::WaylandWithXwayland)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPidBackend {
    Kwin,
    GnomeShellEval,
    X11,
}

pub fn detect_desktop_session() -> DesktopSession {
    detect_desktop_session_from(
        std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(),
        std::env::var("XDG_SESSION_DESKTOP").ok().as_deref(),
        std::env::var("DESKTOP_SESSION").ok().as_deref(),
    )
}

fn detect_desktop_session_from(
    xdg_current_desktop: Option<&str>,
    xdg_session_desktop: Option<&str>,
    desktop_session: Option<&str>,
) -> DesktopSession {
    let candidates = [xdg_current_desktop, xdg_session_desktop, desktop_session];
    for raw in candidates.into_iter().flatten() {
        for token in raw.split(':') {
            let token = token.trim().to_ascii_lowercase();
            match token.as_str() {
                "kde" | "plasma" | "kde-plasma" => return DesktopSession::Kde,
                "gnome" | "gnome-classic" | "gnome-xorg" | "ubuntu" | "pop" => {
                    return DesktopSession::Gnome;
                }
                "sway" | "hyprland" | "wlroots" | "cosmic" | "wayfire" | "river" | "niri" => {
                    return DesktopSession::Other;
                }
                _ => {}
            }
        }
    }
    DesktopSession::Other
}

pub fn detect_display_server() -> DisplayServer {
    detect_display_server_from(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    )
}

fn detect_display_server_from(
    xdg_session_type: Option<&str>,
    display: Option<&str>,
    wayland_display: Option<&str>,
) -> DisplayServer {
    let has_x11 = display.is_some_and(|v| !v.is_empty());
    let has_wayland = wayland_display.is_some_and(|v| !v.is_empty());
    match (has_x11, has_wayland) {
        (true, true) => DisplayServer::WaylandWithXwayland,
        (true, false) => DisplayServer::X11,
        (false, true) => DisplayServer::Wayland,
        (false, false) => match xdg_session_type {
            Some("x11") => DisplayServer::X11,
            Some("wayland") => DisplayServer::Wayland,
            _ => DisplayServer::Unknown,
        },
    }
}

pub fn window_pid_backend_precedence() -> Vec<WindowPidBackend> {
    backend_precedence_for(
        detect_desktop_session(),
        detect_display_server(),
        has_dbus_session(),
    )
}

fn backend_precedence_for(
    session: DesktopSession,
    display: DisplayServer,
    dbus_available: bool,
) -> Vec<WindowPidBackend> {
    let mut out = Vec::with_capacity(3);
    match session {
        DesktopSession::Kde => {
            if dbus_available {
                out.push(WindowPidBackend::Kwin);
            }
            if display.x11_reachable() {
                out.push(WindowPidBackend::X11);
            }
        }
        DesktopSession::Gnome => {
            if dbus_available {
                out.push(WindowPidBackend::GnomeShellEval);
            }
            if display.x11_reachable() {
                out.push(WindowPidBackend::X11);
            }
        }
        DesktopSession::Other => {
            if dbus_available {
                out.push(WindowPidBackend::Kwin);
                out.push(WindowPidBackend::GnomeShellEval);
            }
            if display.x11_reachable() {
                out.push(WindowPidBackend::X11);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_kde_from_xdg_current_desktop() {
        assert_eq!(
            detect_desktop_session_from(Some("KDE"), None, None),
            DesktopSession::Kde
        );
        assert_eq!(
            detect_desktop_session_from(Some("plasma"), None, None),
            DesktopSession::Kde
        );
    }

    #[test]
    fn detect_gnome_handles_colon_list_and_ubuntu_pop_overrides() {
        assert_eq!(
            detect_desktop_session_from(Some("ubuntu:GNOME"), None, None),
            DesktopSession::Gnome
        );
        assert_eq!(
            detect_desktop_session_from(Some("pop:GNOME"), None, None),
            DesktopSession::Gnome
        );
    }

    #[test]
    fn detect_other_for_xfce_and_unset() {
        assert_eq!(
            detect_desktop_session_from(Some("XFCE"), None, None),
            DesktopSession::Other
        );
        assert_eq!(
            detect_desktop_session_from(None, None, None),
            DesktopSession::Other
        );
    }

    #[test]
    fn precedence_kde_session_tries_kwin_first_then_x11() {
        assert_eq!(
            backend_precedence_for(DesktopSession::Kde, DisplayServer::X11, true),
            vec![WindowPidBackend::Kwin, WindowPidBackend::X11]
        );
        assert_eq!(
            backend_precedence_for(DesktopSession::Kde, DisplayServer::Wayland, true),
            vec![WindowPidBackend::Kwin]
        );
    }

    #[test]
    fn precedence_gnome_session_tries_gnome_shell_first_then_x11() {
        assert_eq!(
            backend_precedence_for(
                DesktopSession::Gnome,
                DisplayServer::WaylandWithXwayland,
                true
            ),
            vec![WindowPidBackend::GnomeShellEval, WindowPidBackend::X11]
        );
    }

    #[test]
    fn precedence_unknown_de_tries_all_three_in_order() {
        assert_eq!(
            backend_precedence_for(DesktopSession::Other, DisplayServer::X11, true),
            vec![
                WindowPidBackend::Kwin,
                WindowPidBackend::GnomeShellEval,
                WindowPidBackend::X11,
            ]
        );
    }

    #[test]
    fn precedence_unknown_de_pure_wayland_skips_x11() {
        assert_eq!(
            backend_precedence_for(DesktopSession::Other, DisplayServer::Wayland, true),
            vec![WindowPidBackend::Kwin, WindowPidBackend::GnomeShellEval]
        );
    }

    #[test]
    fn precedence_without_dbus_skips_all_dbus_backends() {
        assert_eq!(
            backend_precedence_for(DesktopSession::Kde, DisplayServer::X11, false),
            vec![WindowPidBackend::X11]
        );
        assert_eq!(
            backend_precedence_for(
                DesktopSession::Gnome,
                DisplayServer::WaylandWithXwayland,
                false
            ),
            vec![WindowPidBackend::X11]
        );
        assert_eq!(
            backend_precedence_for(DesktopSession::Other, DisplayServer::X11, false),
            vec![WindowPidBackend::X11]
        );
    }

    #[test]
    fn precedence_headless_container_returns_empty_list() {
        assert_eq!(
            backend_precedence_for(DesktopSession::Other, DisplayServer::Unknown, false),
            Vec::<WindowPidBackend>::new()
        );
        assert_eq!(
            backend_precedence_for(DesktopSession::Gnome, DisplayServer::Wayland, false),
            Vec::<WindowPidBackend>::new()
        );
    }

    #[test]
    fn wlroots_compositors_bucket_as_other() {
        for token in [
            "sway", "Hyprland", "wlroots", "cosmic", "wayfire", "river", "niri",
        ] {
            assert_eq!(
                detect_desktop_session_from(Some(token), None, None),
                DesktopSession::Other,
                "expected {token} to bucket as Other",
            );
        }
    }

    #[test]
    fn wlroots_compositor_precedence_skips_kwin_and_gnome_shell_when_no_dbus() {
        assert_eq!(
            backend_precedence_for(
                detect_desktop_session_from(Some("sway"), None, None),
                DisplayServer::Wayland,
                false,
            ),
            Vec::<WindowPidBackend>::new()
        );
        assert_eq!(
            backend_precedence_for(
                detect_desktop_session_from(Some("Hyprland"), None, None),
                DisplayServer::WaylandWithXwayland,
                true,
            ),
            vec![
                WindowPidBackend::Kwin,
                WindowPidBackend::GnomeShellEval,
                WindowPidBackend::X11,
            ]
        );
    }

    #[test]
    fn dbus_session_detected_via_bus_address() {
        assert!(has_dbus_session_from(
            Some("unix:path=/run/user/1000/bus"),
            None,
            |_| false
        ));
    }

    #[test]
    fn dbus_session_detected_via_xdg_runtime_dir_socket() {
        assert!(has_dbus_session_from(
            None,
            Some("/run/user/1000"),
            |path| path == "/run/user/1000/bus"
        ));
        assert!(has_dbus_session_from(
            None,
            Some("/run/user/1000/"),
            |path| path == "/run/user/1000/bus"
        ));
    }

    #[test]
    fn dbus_session_absent_when_neither_var_set() {
        assert!(!has_dbus_session_from(None, None, |_| false));
        assert!(!has_dbus_session_from(Some(""), Some(""), |_| true));
    }

    #[test]
    fn dbus_session_absent_when_runtime_dir_has_no_bus_socket() {
        assert!(!has_dbus_session_from(None, Some("/tmp/xdg"), |_| false));
    }

    #[test]
    fn display_server_xwayland_counts_as_x11_reachable() {
        let ds = detect_display_server_from(Some("wayland"), Some(":0"), Some("wayland-0"));
        assert_eq!(ds, DisplayServer::WaylandWithXwayland);
        assert!(ds.x11_reachable());
    }

    #[test]
    fn display_server_pure_wayland_blocks_x11() {
        let ds = detect_display_server_from(Some("wayland"), None, Some("wayland-0"));
        assert!(!ds.x11_reachable());
    }
}
