// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::events::{JsonValue, json_object};
#[cfg(feature = "publisher")]
use livekit::AudioProcessingOptions;

pub const MIN_VOLUME: f64 = 0.0;
pub const MAX_VOLUME: f64 = 2.0;

pub fn clamp_volume(volume: f64) -> f64 {
    if !volume.is_finite() {
        return 1.0;
    }
    volume.clamp(MIN_VOLUME, MAX_VOLUME)
}

pub fn is_muted_volume(volume: f64) -> bool {
    clamp_volume(volume) <= f64::EPSILON
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioDeviceRole {
    Default,
    Communications,
    Endpoint,
}

impl AudioDeviceRole {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Communications => "communications",
            Self::Endpoint => "endpoint",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutputDevice {
    pub device_id: String,
    pub label: String,
    pub is_default: bool,
    pub role: AudioDeviceRole,
    pub endpoint_label: String,
    pub is_default_route: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioInputDevice {
    pub device_id: String,
    pub label: String,
    pub is_default: bool,
    pub role: AudioDeviceRole,
    pub endpoint_label: String,
    pub is_default_route: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShapedAudioDevice {
    device_id: String,
    label: String,
    is_default: bool,
    role: AudioDeviceRole,
    endpoint_label: String,
    is_default_route: bool,
}

const DEFAULT_AUDIO_ROUTE_LABEL: &str = "Default";

fn default_output_device() -> AudioOutputDevice {
    AudioOutputDevice {
        device_id: "default".to_string(),
        label: DEFAULT_AUDIO_ROUTE_LABEL.to_string(),
        is_default: true,
        role: AudioDeviceRole::Default,
        endpoint_label: DEFAULT_AUDIO_ROUTE_LABEL.to_string(),
        is_default_route: true,
    }
}

fn default_input_device() -> AudioInputDevice {
    AudioInputDevice {
        device_id: "default".to_string(),
        label: DEFAULT_AUDIO_ROUTE_LABEL.to_string(),
        is_default: true,
        role: AudioDeviceRole::Default,
        endpoint_label: DEFAULT_AUDIO_ROUTE_LABEL.to_string(),
        is_default_route: true,
    }
}

pub fn output_device_json(device: &AudioOutputDevice) -> String {
    json_object(&[
        ("deviceId", JsonValue::Str(device.device_id.clone())),
        ("label", JsonValue::Str(device.label.clone())),
        (
            "isDefault",
            JsonValue::Raw(if device.is_default { "true" } else { "false" }.to_string()),
        ),
        ("role", JsonValue::Str(device.role.as_str().to_string())),
        (
            "endpointLabel",
            JsonValue::Str(device.endpoint_label.clone()),
        ),
        (
            "isDefaultRoute",
            JsonValue::Raw(
                if device.is_default_route {
                    "true"
                } else {
                    "false"
                }
                .to_string(),
            ),
        ),
    ])
}

pub fn input_device_json(device: &AudioInputDevice) -> String {
    json_object(&[
        ("deviceId", JsonValue::Str(device.device_id.clone())),
        ("label", JsonValue::Str(device.label.clone())),
        (
            "isDefault",
            JsonValue::Raw(if device.is_default { "true" } else { "false" }.to_string()),
        ),
        ("role", JsonValue::Str(device.role.as_str().to_string())),
        (
            "endpointLabel",
            JsonValue::Str(device.endpoint_label.clone()),
        ),
        (
            "isDefaultRoute",
            JsonValue::Raw(
                if device.is_default_route {
                    "true"
                } else {
                    "false"
                }
                .to_string(),
            ),
        ),
    ])
}

pub fn output_devices_json(devices: &[AudioOutputDevice]) -> String {
    let items: Vec<String> = devices.iter().map(output_device_json).collect();
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(item);
    }
    out.push(']');
    out
}

pub fn default_output_devices_json() -> String {
    output_devices_json(&[default_output_device()])
}

pub fn input_devices_json(devices: &[AudioInputDevice]) -> String {
    let items: Vec<String> = devices.iter().map(input_device_json).collect();
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(item);
    }
    out.push(']');
    out
}

pub fn default_input_devices_json() -> String {
    input_devices_json(&[default_input_device()])
}

pub fn shape_output_devices(raw: &[(String, String, usize)]) -> Vec<AudioOutputDevice> {
    shape_audio_devices(raw)
        .into_iter()
        .map(|device| AudioOutputDevice {
            device_id: device.device_id,
            label: device.label,
            is_default: device.is_default,
            role: device.role,
            endpoint_label: device.endpoint_label,
            is_default_route: device.is_default_route,
        })
        .collect()
}

pub fn shape_input_devices(raw: &[(String, String, usize)]) -> Vec<AudioInputDevice> {
    shape_audio_devices(raw)
        .into_iter()
        .map(|device| AudioInputDevice {
            device_id: device.device_id,
            label: device.label,
            is_default: device.is_default,
            role: device.role,
            endpoint_label: device.endpoint_label,
            is_default_route: device.is_default_route,
        })
        .collect()
}

pub const MAX_PLATFORM_AUDIO_DEVICES: usize = 64;
pub const SHAPED_AUDIO_DEVICES_MAX: usize = MAX_PLATFORM_AUDIO_DEVICES + 1;

const _: () = assert!(SHAPED_AUDIO_DEVICES_MAX > MAX_PLATFORM_AUDIO_DEVICES);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayoutSwitchPlan {
    ColdSelect,
    HotSwap,
}

pub fn bounded_audio_device_count(reported: i16) -> Result<usize, String> {
    if reported < 0 {
        return Err(format!(
            "ADM reported negative audio device count: {reported}"
        ));
    }
    let count = (reported as usize).min(MAX_PLATFORM_AUDIO_DEVICES);
    assert!(count <= MAX_PLATFORM_AUDIO_DEVICES);
    Ok(count)
}

pub fn playout_switch_plan(
    platform_playout_active: bool,
    playout_initialized: bool,
) -> PlayoutSwitchPlan {
    if platform_playout_active && playout_initialized {
        return PlayoutSwitchPlan::HotSwap;
    }
    PlayoutSwitchPlan::ColdSelect
}

pub fn resolve_playout_device_guid(
    requested: &str,
    raw: &[(String, String, usize)],
) -> Result<String, String> {
    let requested = requested.trim();
    if requested.is_empty() || requested == "default" {
        return find_default_device_id(raw)
            .ok_or_else(|| "no audio output devices available".to_string());
    }
    let found = raw
        .iter()
        .find(|(id, _, _)| id.trim() == requested)
        .map(|(id, _, _)| id.trim().to_string());
    found.ok_or_else(|| format!("audio output device not found: {requested}"))
}

fn shape_audio_devices(raw: &[(String, String, usize)]) -> Vec<ShapedAudioDevice> {
    let mut devices: Vec<ShapedAudioDevice> = Vec::with_capacity(SHAPED_AUDIO_DEVICES_MAX);
    for (id, label, index) in raw.iter().filter(|(id, _, _)| !id.trim().is_empty()) {
        if devices.len() == SHAPED_AUDIO_DEVICES_MAX {
            break;
        }
        let shaped = shape_audio_device(id, label, *index);
        let duplicate = devices
            .iter()
            .any(|existing| existing.device_id == shaped.device_id);
        if !duplicate {
            devices.push(shaped);
        }
    }
    if !devices.iter().any(|device| device.is_default_route)
        && devices.len() < SHAPED_AUDIO_DEVICES_MAX
        && let Some(default_endpoint) = devices.iter().find(|device| {
            device.is_default
                && device.role == AudioDeviceRole::Endpoint
                && !device.endpoint_label.is_empty()
        })
    {
        devices.insert(
            0,
            ShapedAudioDevice {
                device_id: "default".to_string(),
                label: default_endpoint.endpoint_label.clone(),
                is_default: true,
                role: AudioDeviceRole::Default,
                endpoint_label: default_endpoint.endpoint_label.clone(),
                is_default_route: true,
            },
        );
    }
    assert!(devices.len() <= SHAPED_AUDIO_DEVICES_MAX);
    assert!(
        devices
            .iter()
            .filter(|device| device.is_default_route)
            .count()
            <= 1
    );
    devices
}

fn shape_audio_device(id: &str, raw_label: &str, index: usize) -> ShapedAudioDevice {
    let id = id.trim();
    assert!(!id.is_empty());
    let label = strip_usb_hardware_id_suffix(raw_label);
    if let Some(endpoint_label) = default_route_endpoint_label(id, &label) {
        return ShapedAudioDevice {
            device_id: "default".to_string(),
            label: endpoint_label.clone(),
            is_default: true,
            role: AudioDeviceRole::Default,
            endpoint_label,
            is_default_route: true,
        };
    }
    if let Some(endpoint_label) = communications_route_endpoint_label(id, &label) {
        return ShapedAudioDevice {
            device_id: id.to_string(),
            label: endpoint_label.clone(),
            is_default: index == 0,
            role: AudioDeviceRole::Communications,
            endpoint_label,
            is_default_route: false,
        };
    }
    ShapedAudioDevice {
        device_id: id.to_string(),
        label: label.clone(),
        is_default: index == 0,
        role: AudioDeviceRole::Endpoint,
        endpoint_label: label,
        is_default_route: false,
    }
}

fn default_route_endpoint_label(id: &str, label: &str) -> Option<String> {
    let wrapped = strip_windows_role_prefix(label, "Default")
        .or_else(|| strip_adm_default_wrapper(label))
        .or_else(|| strip_pulse_default_prefix(label));
    if let Some(endpoint_label) = wrapped {
        return Some(strip_usb_hardware_id_suffix(&endpoint_label));
    }
    if id.eq_ignore_ascii_case("default") {
        if label.eq_ignore_ascii_case("default") {
            return Some(String::new());
        }
        return Some(label.to_string());
    }
    None
}

fn communications_route_endpoint_label(id: &str, label: &str) -> Option<String> {
    let wrapped = strip_windows_role_prefix(label, "Communications")
        .or_else(|| strip_windows_role_prefix(label, "Communication"));
    if let Some(endpoint_label) = wrapped {
        return Some(strip_usb_hardware_id_suffix(&endpoint_label));
    }
    if id.eq_ignore_ascii_case("communications") {
        return Some(label.to_string());
    }
    None
}

pub fn is_default_route_device(id: &str, label: &str) -> bool {
    let label = strip_usb_hardware_id_suffix(label);
    default_route_endpoint_label(id.trim(), &label).is_some()
}

pub fn find_default_device_id(raw: &[(String, String, usize)]) -> Option<String> {
    let default_route = raw
        .iter()
        .find(|(id, label, _)| !id.trim().is_empty() && is_default_route_device(id, label));
    if let Some((id, _, _)) = default_route {
        return Some(id.trim().to_string());
    }
    raw.iter()
        .find(|(id, _, _)| !id.trim().is_empty())
        .map(|(id, _, _)| id.trim().to_string())
}

fn strip_windows_role_prefix(label: &str, role: &str) -> Option<String> {
    let (prefix, endpoint) = label.trim().split_once('-')?;
    if prefix.trim().eq_ignore_ascii_case(role) {
        let endpoint = endpoint.trim();
        if !endpoint.is_empty() {
            return Some(endpoint.to_string());
        }
    }
    None
}

fn strip_adm_default_wrapper(label: &str) -> Option<String> {
    const WRAPPER_PREFIX: &str = "default (";
    let trimmed = label.trim();
    if trimmed.len() < WRAPPER_PREFIX.len() + 1 {
        return None;
    }
    if !trimmed.is_char_boundary(WRAPPER_PREFIX.len()) {
        return None;
    }
    if !trimmed[..WRAPPER_PREFIX.len()].eq_ignore_ascii_case(WRAPPER_PREFIX) {
        return None;
    }
    if !trimmed.ends_with(')') {
        return None;
    }
    let inner = trimmed[WRAPPER_PREFIX.len()..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_string())
}

fn strip_pulse_default_prefix(label: &str) -> Option<String> {
    let inner = label.trim().strip_prefix("default: ")?.trim();
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_string())
}

fn strip_usb_hardware_id_suffix(label: &str) -> String {
    let trimmed = label.trim();
    let Some(open_index) = trimmed.rfind('(') else {
        return trimmed.to_string();
    };
    if !trimmed.ends_with(')') || open_index + 1 >= trimmed.len() {
        return trimmed.to_string();
    }
    let suffix = &trimmed[open_index + 1..trimmed.len() - 1];
    if suffix.len() != 9 {
        return trimmed.to_string();
    }
    let suffix_bytes = suffix.as_bytes();
    if suffix_bytes.get(4) != Some(&b':') {
        return trimmed.to_string();
    }
    let is_hardware_id = suffix_bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || byte.is_ascii_hexdigit());
    if !is_hardware_id {
        return trimmed.to_string();
    }
    trimmed[..open_index].trim_end().to_string()
}

pub const MICROPHONE_MAX_BITRATE_BPS_FLOOR: u64 = 8_000;
pub const MICROPHONE_MAX_BITRATE_BPS_CAP: u64 = 510_000;

pub fn normalize_microphone_max_bitrate_bps(
    max_bitrate_bps: Option<f64>,
) -> Result<Option<u64>, String> {
    let Some(raw) = max_bitrate_bps else {
        return Ok(None);
    };
    if !raw.is_finite() {
        return Err("microphone maxBitrateBps must be finite".to_string());
    }
    if raw <= 0.0 {
        return Err("microphone maxBitrateBps must be positive".to_string());
    }
    if raw.fract() != 0.0 {
        return Err("microphone maxBitrateBps must be an integer".to_string());
    }
    let clamped = (raw as u64).clamp(
        MICROPHONE_MAX_BITRATE_BPS_FLOOR,
        MICROPHONE_MAX_BITRATE_BPS_CAP,
    );
    assert!(clamped >= MICROPHONE_MAX_BITRATE_BPS_FLOOR);
    assert!(clamped <= MICROPHONE_MAX_BITRATE_BPS_CAP);
    Ok(Some(clamped))
}

#[cfg(feature = "publisher")]
pub fn processing_options(
    echo_cancellation: Option<bool>,
    noise_suppression: Option<bool>,
    auto_gain_control: Option<bool>,
) -> AudioProcessingOptions {
    let mut options = AudioProcessingOptions::default();
    if let Some(enabled) = echo_cancellation {
        options.echo_cancellation = enabled;
    }
    if let Some(enabled) = noise_suppression {
        options.noise_suppression = enabled;
    }
    if let Some(enabled) = auto_gain_control {
        options.auto_gain_control = enabled;
    }
    options
}

pub const DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN: f64 = 0.0;
pub const DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX: f64 = 100.0;

const _: () =
    assert!(DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN < DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX);

pub fn clamp_deep_filter_noise_reduction_level(level: f64) -> f64 {
    if !level.is_finite() {
        return DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX;
    }
    let clamped = level.clamp(
        DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN,
        DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX,
    );
    assert!(clamped >= DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN);
    assert!(clamped <= DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX);
    clamped
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MicrophoneApmIntent {
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
}

pub fn resolve_microphone_apm_intent(
    echo_cancellation: Option<bool>,
    noise_suppression: Option<bool>,
    auto_gain_control: Option<bool>,
    deep_filter_requested: bool,
    native_deep_filter_available: bool,
) -> MicrophoneApmIntent {
    if !deep_filter_requested {
        return MicrophoneApmIntent {
            echo_cancellation,
            noise_suppression,
            auto_gain_control,
        };
    }
    if native_deep_filter_available {
        return MicrophoneApmIntent {
            echo_cancellation,
            noise_suppression: Some(false),
            auto_gain_control: Some(false),
        };
    }
    MicrophoneApmIntent {
        echo_cancellation,
        noise_suppression: Some(true),
        auto_gain_control: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_volume_holds_the_contract_range() {
        assert_eq!(clamp_volume(1.0), 1.0);
        assert_eq!(clamp_volume(0.0), 0.0);
        assert_eq!(clamp_volume(2.0), 2.0);
        assert_eq!(clamp_volume(-1.0), 0.0);
        assert_eq!(clamp_volume(5.0), 2.0);
        assert_eq!(clamp_volume(2.5), 2.0);
        assert_eq!(clamp_volume(0.5), 0.5);
        assert_eq!(clamp_volume(1.75), 1.75);
    }

    #[test]
    fn clamp_volume_maps_non_finite_to_unity() {
        assert_eq!(clamp_volume(f64::NAN), 1.0);
        assert_eq!(clamp_volume(f64::INFINITY), 1.0);
        assert_eq!(clamp_volume(f64::NEG_INFINITY), 1.0);
    }

    #[test]
    fn microphone_max_bitrate_passes_through_in_range_values() {
        assert_eq!(normalize_microphone_max_bitrate_bps(None), Ok(None));
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(8_000.0)),
            Ok(Some(8_000))
        );
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(96_000.0)),
            Ok(Some(96_000))
        );
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(510_000.0)),
            Ok(Some(510_000))
        );
    }

    #[test]
    fn microphone_max_bitrate_clamps_to_floor_and_cap() {
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(1.0)),
            Ok(Some(MICROPHONE_MAX_BITRATE_BPS_FLOOR))
        );
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(7_999.0)),
            Ok(Some(MICROPHONE_MAX_BITRATE_BPS_FLOOR))
        );
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(510_001.0)),
            Ok(Some(MICROPHONE_MAX_BITRATE_BPS_CAP))
        );
        assert_eq!(
            normalize_microphone_max_bitrate_bps(Some(1_000_000.0)),
            Ok(Some(MICROPHONE_MAX_BITRATE_BPS_CAP))
        );
    }

    #[test]
    fn microphone_max_bitrate_rejects_invalid_values() {
        assert!(normalize_microphone_max_bitrate_bps(Some(f64::NAN)).is_err());
        assert!(normalize_microphone_max_bitrate_bps(Some(f64::INFINITY)).is_err());
        assert!(normalize_microphone_max_bitrate_bps(Some(f64::NEG_INFINITY)).is_err());
        assert!(normalize_microphone_max_bitrate_bps(Some(0.0)).is_err());
        assert!(normalize_microphone_max_bitrate_bps(Some(-64_000.0)).is_err());
        assert!(normalize_microphone_max_bitrate_bps(Some(64_000.5)).is_err());
    }

    #[test]
    fn is_muted_only_at_zero() {
        assert!(is_muted_volume(0.0));
        assert!(is_muted_volume(-1.0));
        assert!(!is_muted_volume(0.01));
        assert!(!is_muted_volume(1.0));
        assert!(!is_muted_volume(2.0));
    }

    #[test]
    fn bounded_audio_device_count_rejects_negative_counts() {
        assert!(bounded_audio_device_count(-1).is_err());
        assert!(bounded_audio_device_count(i16::MIN).is_err());
    }

    #[test]
    fn bounded_audio_device_count_clamps_to_the_platform_cap() {
        assert_eq!(bounded_audio_device_count(0), Ok(0));
        assert_eq!(bounded_audio_device_count(3), Ok(3));
        assert_eq!(
            bounded_audio_device_count((MAX_PLATFORM_AUDIO_DEVICES + 1) as i16),
            Ok(MAX_PLATFORM_AUDIO_DEVICES)
        );
        assert_eq!(
            bounded_audio_device_count(i16::MAX),
            Ok(MAX_PLATFORM_AUDIO_DEVICES)
        );
    }

    #[test]
    fn playout_switch_plan_only_hot_swaps_live_platform_playout() {
        assert_eq!(
            playout_switch_plan(false, false),
            PlayoutSwitchPlan::ColdSelect
        );
        assert_eq!(
            playout_switch_plan(true, false),
            PlayoutSwitchPlan::ColdSelect
        );
        assert_eq!(
            playout_switch_plan(false, true),
            PlayoutSwitchPlan::ColdSelect
        );
        assert_eq!(playout_switch_plan(true, true), PlayoutSwitchPlan::HotSwap);
    }

    #[test]
    fn resolve_playout_device_guid_resolves_default_to_the_route_guid() {
        let raw = vec![
            (
                "default".to_string(),
                "default (Studio Display)".to_string(),
                0,
            ),
            ("speaker-guid".to_string(), "Studio Display".to_string(), 1),
        ];

        assert_eq!(
            resolve_playout_device_guid("", &raw),
            Ok("default".to_string())
        );
        assert_eq!(
            resolve_playout_device_guid("default", &raw),
            Ok("default".to_string())
        );
    }

    #[test]
    fn resolve_playout_device_guid_rejects_default_when_no_devices_exist() {
        let raw: Vec<(String, String, usize)> = vec![];

        assert!(resolve_playout_device_guid("default", &raw).is_err());
    }

    #[test]
    fn resolve_playout_device_guid_prevalidates_explicit_guids() {
        let raw = vec![
            ("speaker-guid".to_string(), "Studio Display".to_string(), 0),
            ("hdmi-guid".to_string(), "HDMI".to_string(), 1),
        ];

        assert_eq!(
            resolve_playout_device_guid(" speaker-guid ", &raw),
            Ok("speaker-guid".to_string())
        );
        assert!(resolve_playout_device_guid("missing-guid", &raw).is_err());
    }

    #[test]
    fn output_device_json_locks_the_contract_shape() {
        let dev = AudioOutputDevice {
            device_id: "{0.0.0.00000000}.{guid}".into(),
            label: "Speakers (Realtek)".into(),
            is_default: true,
            role: AudioDeviceRole::Endpoint,
            endpoint_label: "Speakers (Realtek)".into(),
            is_default_route: false,
        };
        assert_eq!(
            output_device_json(&dev),
            "{\"deviceId\":\"{0.0.0.00000000}.{guid}\",\"label\":\"Speakers (Realtek)\",\"isDefault\":true,\"role\":\"endpoint\",\"endpointLabel\":\"Speakers (Realtek)\",\"isDefaultRoute\":false}"
        );
    }

    #[test]
    fn output_devices_json_array() {
        assert_eq!(output_devices_json(&[]), "[]");
        let list = vec![
            AudioOutputDevice {
                device_id: "a".into(),
                label: "A".into(),
                is_default: true,
                role: AudioDeviceRole::Default,
                endpoint_label: "A".into(),
                is_default_route: true,
            },
            AudioOutputDevice {
                device_id: "b".into(),
                label: "B".into(),
                is_default: false,
                role: AudioDeviceRole::Endpoint,
                endpoint_label: "B".into(),
                is_default_route: false,
            },
        ];
        assert_eq!(
            output_devices_json(&list),
            "[{\"deviceId\":\"a\",\"label\":\"A\",\"isDefault\":true,\"role\":\"default\",\"endpointLabel\":\"A\",\"isDefaultRoute\":true},{\"deviceId\":\"b\",\"label\":\"B\",\"isDefault\":false,\"role\":\"endpoint\",\"endpointLabel\":\"B\",\"isDefaultRoute\":false}]"
        );
    }

    #[test]
    fn default_output_devices_json_exposes_a_default_route_placeholder() {
        assert_eq!(
            default_output_devices_json(),
            "[{\"deviceId\":\"default\",\"label\":\"Default\",\"isDefault\":true,\"role\":\"default\",\"endpointLabel\":\"Default\",\"isDefaultRoute\":true}]"
        );
    }

    #[test]
    fn input_devices_json_array() {
        assert_eq!(input_devices_json(&[]), "[]");
        let list = vec![
            AudioInputDevice {
                device_id: "mic-a".into(),
                label: "Built-in Microphone".into(),
                is_default: true,
                role: AudioDeviceRole::Default,
                endpoint_label: "Built-in Microphone".into(),
                is_default_route: true,
            },
            AudioInputDevice {
                device_id: "mic-b".into(),
                label: "USB Mic".into(),
                is_default: false,
                role: AudioDeviceRole::Endpoint,
                endpoint_label: "USB Mic".into(),
                is_default_route: false,
            },
        ];
        assert_eq!(
            input_devices_json(&list),
            "[{\"deviceId\":\"mic-a\",\"label\":\"Built-in Microphone\",\"isDefault\":true,\"role\":\"default\",\"endpointLabel\":\"Built-in Microphone\",\"isDefaultRoute\":true},{\"deviceId\":\"mic-b\",\"label\":\"USB Mic\",\"isDefault\":false,\"role\":\"endpoint\",\"endpointLabel\":\"USB Mic\",\"isDefaultRoute\":false}]"
        );
    }

    #[test]
    fn default_input_devices_json_exposes_a_default_route_placeholder() {
        assert_eq!(
            default_input_devices_json(),
            "[{\"deviceId\":\"default\",\"label\":\"Default\",\"isDefault\":true,\"role\":\"default\",\"endpointLabel\":\"Default\",\"isDefaultRoute\":true}]"
        );
    }

    #[test]
    fn shape_adds_default_route_for_index_zero_endpoint_and_drops_empty_ids() {
        let raw = vec![
            (
                "default-guid".to_string(),
                "Default Speakers".to_string(),
                0,
            ),
            ("hdmi-guid".to_string(), "HDMI Out".to_string(), 1),
            (String::new(), "Phantom".to_string(), 2),
        ];
        let shaped = shape_output_devices(&raw);
        assert_eq!(shaped.len(), 3);
        assert!(shaped[0].is_default);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(shaped[0].role, AudioDeviceRole::Default);
        assert!(shaped[0].is_default_route);
        assert!(shaped[1].is_default);
        assert_eq!(shaped[1].device_id, "default-guid");
        assert_eq!(shaped[1].role, AudioDeviceRole::Endpoint);
        assert!(!shaped[2].is_default);
        assert_eq!(shaped[2].label, "HDMI Out");
    }

    #[test]
    fn shape_input_adds_default_route_for_index_zero_endpoint_and_drops_empty_ids() {
        let raw = vec![
            ("default-mic".to_string(), "Default Mic".to_string(), 0),
            ("usb-mic".to_string(), "USB Mic".to_string(), 1),
            (String::new(), "Phantom".to_string(), 2),
        ];
        let shaped = shape_input_devices(&raw);
        assert_eq!(shaped.len(), 3);
        assert!(shaped[0].is_default);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(shaped[0].role, AudioDeviceRole::Default);
        assert!(shaped[0].is_default_route);
        assert!(shaped[1].is_default);
        assert_eq!(shaped[1].device_id, "default-mic");
        assert_eq!(shaped[1].role, AudioDeviceRole::Endpoint);
        assert!(!shaped[2].is_default);
        assert_eq!(shaped[2].label, "USB Mic");
    }

    #[test]
    fn shape_recognizes_macos_virtual_default_without_duplicating_it() {
        let raw = vec![
            ("default".to_string(), "default (WH-1000XM5)".to_string(), 0),
            ("74".to_string(), "Studio Display Microphone".to_string(), 1),
            ("81".to_string(), "WH-1000XM5".to_string(), 2),
        ];
        let shaped = shape_input_devices(&raw);
        assert_eq!(shaped.len(), 3);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(shaped[0].label, "WH-1000XM5");
        assert_eq!(shaped[0].endpoint_label, "WH-1000XM5");
        assert_eq!(shaped[0].role, AudioDeviceRole::Default);
        assert!(shaped[0].is_default_route);
        assert_eq!(shaped[1].device_id, "74");
        assert_eq!(shaped[1].role, AudioDeviceRole::Endpoint);
        assert_eq!(shaped[2].device_id, "81");
        let default_route_count = shaped
            .iter()
            .filter(|device| device.is_default_route)
            .count();
        assert_eq!(default_route_count, 1);
    }

    #[test]
    fn shape_collapses_duplicate_device_ids() {
        let raw = vec![
            ("default".to_string(), "default (WH-1000XM5)".to_string(), 0),
            ("default".to_string(), "default (WH-1000XM5)".to_string(), 1),
            ("81".to_string(), "WH-1000XM5".to_string(), 2),
            ("81".to_string(), "WH-1000XM5".to_string(), 3),
        ];
        let shaped = shape_output_devices(&raw);
        assert_eq!(shaped.len(), 2);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(shaped[1].device_id, "81");
    }

    #[test]
    fn shape_recognizes_pulse_virtual_default_prefix() {
        let raw = vec![
            (
                "pulse-default".to_string(),
                "default: alsa_output.pci-0000_00_1f.3.analog-stereo".to_string(),
                0,
            ),
            (
                "pulse-sink".to_string(),
                "Built-in Audio Analog Stereo".to_string(),
                1,
            ),
        ];
        let shaped = shape_output_devices(&raw);
        assert_eq!(shaped.len(), 2);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(
            shaped[0].label,
            "alsa_output.pci-0000_00_1f.3.analog-stereo"
        );
        assert!(shaped[0].is_default_route);
        assert_eq!(shaped[1].role, AudioDeviceRole::Endpoint);
    }

    #[test]
    fn shape_recognizes_windows_adm2_communication_singular_prefix() {
        let raw = vec![
            (
                "{0.0.1.00000000}.{guid-a}".to_string(),
                "Default - Headset Microphone (2- Arctis 7 Chat)".to_string(),
                0,
            ),
            (
                "{0.0.1.00000000}.{guid-a}".to_string(),
                "Communication - Headset Microphone (2- Arctis 7 Chat)".to_string(),
                1,
            ),
            (
                "{0.0.1.00000000}.{guid-b}".to_string(),
                "Headset Microphone (2- Arctis 7 Chat)".to_string(),
                2,
            ),
        ];
        let shaped = shape_input_devices(&raw);
        assert_eq!(shaped.len(), 3);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(shaped[0].label, "Headset Microphone (2- Arctis 7 Chat)");
        assert_eq!(shaped[1].role, AudioDeviceRole::Communications);
        assert_eq!(shaped[2].role, AudioDeviceRole::Endpoint);
    }

    #[test]
    fn find_default_device_id_prefers_the_virtual_default_route() {
        let macos = vec![
            ("default".to_string(), "default (WH-1000XM5)".to_string(), 0),
            ("81".to_string(), "WH-1000XM5".to_string(), 1),
        ];
        assert_eq!(find_default_device_id(&macos), Some("default".to_string()));

        let no_virtual_default = vec![
            ("guid-a".to_string(), "Speakers (Realtek)".to_string(), 0),
            ("guid-b".to_string(), "HDMI Out".to_string(), 1),
        ];
        assert_eq!(
            find_default_device_id(&no_virtual_default),
            Some("guid-a".to_string())
        );

        let empty: Vec<(String, String, usize)> = vec![];
        assert_eq!(find_default_device_id(&empty), None);
        let only_empty_ids = vec![(String::new(), "Phantom".to_string(), 0)];
        assert_eq!(find_default_device_id(&only_empty_ids), None);
    }

    #[test]
    fn adm_default_wrapper_requires_the_full_shape() {
        assert_eq!(
            strip_adm_default_wrapper("default (WH-1000XM5)"),
            Some("WH-1000XM5".to_string())
        );
        assert_eq!(
            strip_adm_default_wrapper("Default (MacBook Pro Speakers)"),
            Some("MacBook Pro Speakers".to_string())
        );
        assert_eq!(strip_adm_default_wrapper("default ()"), None);
        assert_eq!(strip_adm_default_wrapper("default"), None);
        assert_eq!(strip_adm_default_wrapper("default (unterminated"), None);
        assert_eq!(strip_adm_default_wrapper("WH-1000XM5"), None);
    }

    #[test]
    fn shape_uses_windows_default_and_communications_roles() {
        let raw = vec![
            (
                "default-guid".to_string(),
                "Default - Microphone (3- Logitech PRO X Wireless Gaming Headset) (046d:0aba)".to_string(),
                0,
            ),
            (
                "communications-guid".to_string(),
                "Communications - Microphone (3- Logitech PRO X Wireless Gaming Headset) (046D:0ABA)".to_string(),
                1,
            ),
            (
                "endpoint-guid".to_string(),
                "Microphone (3- Logitech PRO X Wireless Gaming Headset) (046d:0aba)".to_string(),
                2,
            ),
        ];
        let shaped = shape_input_devices(&raw);
        assert_eq!(shaped.len(), 3);
        assert_eq!(shaped[0].device_id, "default");
        assert_eq!(
            shaped[0].label,
            "Microphone (3- Logitech PRO X Wireless Gaming Headset)"
        );
        assert_eq!(shaped[0].role, AudioDeviceRole::Default);
        assert!(shaped[0].is_default_route);
        assert_eq!(
            shaped[1].label,
            "Microphone (3- Logitech PRO X Wireless Gaming Headset)"
        );
        assert_eq!(shaped[1].role, AudioDeviceRole::Communications);
        assert_eq!(
            shaped[2].label,
            "Microphone (3- Logitech PRO X Wireless Gaming Headset)"
        );
        assert_eq!(shaped[2].role, AudioDeviceRole::Endpoint);
    }

    #[test]
    fn clamp_deep_filter_noise_reduction_level_holds_the_contract_range() {
        assert_eq!(clamp_deep_filter_noise_reduction_level(0.0), 0.0);
        assert_eq!(clamp_deep_filter_noise_reduction_level(80.0), 80.0);
        assert_eq!(clamp_deep_filter_noise_reduction_level(100.0), 100.0);
        assert_eq!(clamp_deep_filter_noise_reduction_level(-5.0), 0.0);
        assert_eq!(clamp_deep_filter_noise_reduction_level(150.0), 100.0);
        assert_eq!(clamp_deep_filter_noise_reduction_level(f64::NAN), 100.0);
        assert_eq!(
            clamp_deep_filter_noise_reduction_level(f64::INFINITY),
            100.0
        );
    }

    #[test]
    fn deep_filter_intent_passes_explicit_apm_through_when_not_requested() {
        let intent =
            resolve_microphone_apm_intent(Some(true), Some(false), Some(true), false, false);
        assert_eq!(intent.echo_cancellation, Some(true));
        assert_eq!(intent.noise_suppression, Some(false));
        assert_eq!(intent.auto_gain_control, Some(true));
    }

    #[test]
    fn deep_filter_intent_falls_back_to_apm_when_native_filter_is_unavailable() {
        let intent =
            resolve_microphone_apm_intent(Some(true), Some(false), Some(false), true, false);
        assert_eq!(intent.echo_cancellation, Some(true));
        assert_eq!(intent.noise_suppression, Some(true));
        assert_eq!(intent.auto_gain_control, Some(true));
    }

    #[test]
    fn deep_filter_intent_disables_apm_noise_paths_when_native_filter_runs() {
        let intent = resolve_microphone_apm_intent(Some(true), Some(true), Some(true), true, true);
        assert_eq!(intent.echo_cancellation, Some(true));
        assert_eq!(intent.noise_suppression, Some(false));
        assert_eq!(intent.auto_gain_control, Some(false));
    }

    #[cfg(feature = "publisher")]
    #[test]
    fn processing_options_preserve_defaults_unless_overridden() {
        let defaults = AudioProcessingOptions::default();
        let options = processing_options(Some(false), None, Some(false));
        assert!(!options.echo_cancellation);
        assert_eq!(options.noise_suppression, defaults.noise_suppression);
        assert!(!options.auto_gain_control);
        assert_eq!(
            options.prefer_hardware_processing,
            defaults.prefer_hardware_processing
        );
    }
}
