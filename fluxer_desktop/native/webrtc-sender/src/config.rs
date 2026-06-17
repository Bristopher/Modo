// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishConfig {
    pub url: String,
    pub token: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: String,
}

#[cfg(test)]
impl PublishConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.url.trim().is_empty() {
            return Err("livekit url is empty".into());
        }
        if !(self.url.starts_with("ws://") || self.url.starts_with("wss://")) {
            return Err("livekit url must be ws:// or wss://".into());
        }
        if self.token.trim().is_empty() {
            return Err("livekit token is empty".into());
        }
        if self.width < 2 || self.height < 2 {
            return Err("capture dimensions too small".into());
        }
        if !self.width.is_multiple_of(2) || !self.height.is_multiple_of(2) {
            return Err("capture dimensions must be even".into());
        }
        if self.width > 8192 || self.height > 8192 {
            return Err("capture dimensions too large".into());
        }
        if self.fps == 0 {
            return Err("capture fps must be positive".into());
        }
        if !self.codec.trim().is_empty() && canonical_codec_name(&self.codec).is_none() {
            return Err("unsupported video codec".into());
        }
        Ok(())
    }
}

pub const SUPPORTED_CODECS: &[&str] = &["vp8", "h264", "vp9", "av1", "h265"];

pub fn canonical_codec_name(name: &str) -> Option<&'static str> {
    let lower = name.trim().to_ascii_lowercase();
    if lower == "hevc" {
        return Some("h265");
    }
    SUPPORTED_CODECS.iter().copied().find(|&c| c == lower)
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublisherState {
    Idle,
    Connecting,
    Publishing,
    Closed,
    Failed,
}

#[cfg(test)]
impl PublisherState {
    pub fn accepts_frames(self) -> bool {
        matches!(self, PublisherState::Publishing)
    }

    pub fn can_connect(self) -> bool {
        matches!(
            self,
            PublisherState::Idle | PublisherState::Closed | PublisherState::Failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> PublishConfig {
        PublishConfig {
            url: "wss://sfu.example/rtc".into(),
            token: "jwt".into(),
            width: 1920,
            height: 1080,
            fps: 30,
            codec: String::new(),
        }
    }

    #[test]
    fn valid_config_passes() {
        assert!(cfg().validate().is_ok());
    }

    #[test]
    fn rejects_bad_url() {
        let mut c = cfg();
        c.url = "https://sfu".into();
        assert!(c.validate().is_err());
        c.url = String::new();
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_empty_token() {
        let mut c = cfg();
        c.token = "  ".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_odd_or_oob_dimensions() {
        let mut c = cfg();
        c.width = 1921;
        assert!(c.validate().is_err());
        c.width = 1920;
        c.height = 0;
        assert!(c.validate().is_err());
        c.height = 16384;
        assert!(c.validate().is_err());
    }

    #[test]
    fn accepts_dimension_boundaries_and_rejects_zero_fps() {
        let mut c = cfg();
        c.width = 2;
        c.height = 2;
        c.fps = 1;
        assert!(c.validate().is_ok());

        c.width = 8192;
        c.height = 8192;
        assert!(c.validate().is_ok());

        c.fps = 0;
        assert_eq!(
            c.validate(),
            Err("capture fps must be positive".to_string())
        );
    }

    #[test]
    fn rejects_unknown_non_empty_codec_but_allows_empty_default() {
        let mut c = cfg();
        c.codec = String::new();
        assert!(c.validate().is_ok());

        c.codec = "  ".into();
        assert!(c.validate().is_ok());

        c.codec = "h266".into();
        assert_eq!(c.validate(), Err("unsupported video codec".to_string()));
    }

    #[test]
    fn canonical_codec_name_accepts_all_five_case_insensitively() {
        for (input, expected) in [
            ("vp8", "vp8"),
            (" vp8 ", "vp8"),
            ("VP8", "vp8"),
            ("h264", "h264"),
            ("H264", "h264"),
            ("vp9", "vp9"),
            ("Vp9", "vp9"),
            ("av1", "av1"),
            ("AV1", "av1"),
            ("h265", "h265"),
            ("H265", "h265"),
            ("hevc", "h265"),
            ("HEVC", "h265"),
        ] {
            assert_eq!(canonical_codec_name(input), Some(expected), "codec {input}");
        }
        assert_eq!(SUPPORTED_CODECS.len(), 5);
    }

    #[test]
    fn canonical_codec_name_rejects_empty_and_unknown() {
        assert_eq!(canonical_codec_name(""), None);
        assert_eq!(canonical_codec_name("h266"), None);
        assert_eq!(canonical_codec_name("rubbish"), None);
    }

    #[test]
    fn state_gates_frames_and_connect() {
        assert!(PublisherState::Publishing.accepts_frames());
        assert!(!PublisherState::Connecting.accepts_frames());
        assert!(!PublisherState::Idle.accepts_frames());
        assert!(PublisherState::Idle.can_connect());
        assert!(PublisherState::Closed.can_connect());
        assert!(PublisherState::Failed.can_connect());
        assert!(!PublisherState::Failed.accepts_frames());
        assert!(!PublisherState::Publishing.can_connect());
        assert!(!PublisherState::Connecting.can_connect());
    }
}
