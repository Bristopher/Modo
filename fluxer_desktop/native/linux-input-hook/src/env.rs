// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,

    Wayland,

    WaylandWithXwayland,

    Unknown,
}

impl DisplayServer {
    pub fn supports_global_xrecord(self) -> bool {
        matches!(self, Self::X11)
    }
}

pub fn detect_display_server() -> DisplayServer {
    detect_from(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    )
}

fn detect_from(
    xdg_session_type: Option<&str>,
    display: Option<&str>,
    wayland_display: Option<&str>,
) -> DisplayServer {
    let has_x11 = display.is_some_and(|v| !v.is_empty());
    let has_wayland = wayland_display.is_some_and(|v| !v.is_empty());
    let xdg_is_wayland = xdg_session_type == Some("wayland");
    match (has_x11, has_wayland) {
        (true, true) => DisplayServer::WaylandWithXwayland,
        (true, false) if xdg_is_wayland => DisplayServer::WaylandWithXwayland,
        (true, false) => DisplayServer::X11,
        (false, true) => DisplayServer::Wayland,
        (false, false) => match xdg_session_type {
            Some("x11") => DisplayServer::X11,
            Some("wayland") => DisplayServer::Wayland,
            _ => DisplayServer::Unknown,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_pure_x11_from_display_only() {
        assert_eq!(
            detect_from(Some("x11"), Some(":0"), None),
            DisplayServer::X11
        );
        assert_eq!(detect_from(None, Some(":0"), None), DisplayServer::X11);
    }

    #[test]
    fn detect_pure_wayland_from_wayland_display_only() {
        assert_eq!(
            detect_from(Some("wayland"), None, Some("wayland-0")),
            DisplayServer::Wayland
        );
        assert_eq!(
            detect_from(None, None, Some("wayland-0")),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn detect_xwayland_when_both_sockets_present() {
        let ds = detect_from(Some("wayland"), Some(":0"), Some("wayland-0"));
        assert_eq!(ds, DisplayServer::WaylandWithXwayland);
        assert!(!ds.supports_global_xrecord());
    }

    #[test]
    fn detect_xwayland_from_wayland_session_with_display_only() {
        let ds = detect_from(Some("wayland"), Some(":0"), None);
        assert_eq!(ds, DisplayServer::WaylandWithXwayland);
        assert!(!ds.supports_global_xrecord());
    }

    #[test]
    fn detect_unknown_when_nothing_set() {
        assert_eq!(detect_from(None, None, None), DisplayServer::Unknown);
        assert_eq!(detect_from(Some("tty"), None, None), DisplayServer::Unknown);
    }

    #[test]
    fn detect_empty_display_var_is_ignored() {
        assert_eq!(
            detect_from(Some("wayland"), Some(""), Some("wayland-0")),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn xdg_session_type_fallback_when_sockets_missing() {
        assert_eq!(detect_from(Some("x11"), None, None), DisplayServer::X11);
        assert_eq!(
            detect_from(Some("wayland"), None, None),
            DisplayServer::Wayland
        );
    }

    #[test]
    fn global_xrecord_support_is_only_for_pure_x11() {
        assert!(DisplayServer::X11.supports_global_xrecord());
        assert!(!DisplayServer::WaylandWithXwayland.supports_global_xrecord());
        assert!(!DisplayServer::Wayland.supports_global_xrecord());
        assert!(!DisplayServer::Unknown.supports_global_xrecord());
    }
}
