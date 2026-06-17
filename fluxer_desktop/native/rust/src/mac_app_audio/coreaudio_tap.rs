// SPDX-License-Identifier: AGPL-3.0-or-later

pub type OSStatus = i32;
pub type AudioObjectId = u32;

pub const NO_ERR: OSStatus = 0;
pub const K_AUDIO_OBJECT_UNKNOWN: AudioObjectId = 0;
pub const K_AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectId = 1;
pub const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0;
pub const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = fourcc(*b"glob");
pub const K_AUDIO_HARDWARE_PROPERTY_TRANSLATE_PID_TO_PROCESS_OBJECT: u32 = fourcc(*b"id2p");
pub const K_AUDIO_HARDWARE_PROPERTY_PROCESS_OBJECT_LIST: u32 = fourcc(*b"prs#");
pub const K_AUDIO_TAP_PROPERTY_UID: u32 = fourcc(*b"tuid");
pub const K_AUDIO_TAP_PROPERTY_FORMAT: u32 = fourcc(*b"tfmt");
pub const K_AUDIO_AGGREGATE_DRIFT_COMPENSATION_MEDIUM_QUALITY: u32 = 0x40;
pub const TARGET_SAMPLE_RATE: f64 = 48_000.0;
pub const TARGET_CHANNELS: u32 = 2;
pub const MAX_CALLBACK_INPUT_FRAMES: u32 = 48_000;
pub const MAX_RELATED_PROCESSES: usize = 512;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioObjectPropertyAddress {
    pub selector: u32,
    pub scope: u32,
    pub element: u32,
}

pub const fn fourcc(bytes: [u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | bytes[3] as u32
}

pub fn dedupe_audio_objects(
    objects: impl IntoIterator<Item = AudioObjectId>,
) -> Vec<AudioObjectId> {
    let mut out = Vec::new();
    for object in objects {
        if object == K_AUDIO_OBJECT_UNKNOWN || out.contains(&object) {
            continue;
        }
        out.push(object);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_audio_fourcc_constants_match_headers() {
        assert_eq!(0x676c_6f62, K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL);
        assert_eq!(
            0x6964_3270,
            K_AUDIO_HARDWARE_PROPERTY_TRANSLATE_PID_TO_PROCESS_OBJECT
        );
        assert_eq!(0x7475_6964, K_AUDIO_TAP_PROPERTY_UID);
        assert_eq!(0x7466_6d74, K_AUDIO_TAP_PROPERTY_FORMAT);
    }

    #[test]
    fn core_audio_process_object_collection_keeps_unique_translated_objects() {
        assert_eq!(0, K_AUDIO_OBJECT_UNKNOWN);
        assert_eq!(1, K_AUDIO_OBJECT_SYSTEM_OBJECT);
        let _ = K_AUDIO_HARDWARE_PROPERTY_PROCESS_OBJECT_LIST;
        assert_eq!(vec![7, 8, 9], dedupe_audio_objects([0, 7, 8, 7, 0, 9, 8]));
    }
}
