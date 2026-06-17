// SPDX-License-Identifier: AGPL-3.0-or-later

use napi_derive::napi;

pub const ENGINE_BRIDGE_VERSION: u32 = 18;

const _: () = assert!(ENGINE_BRIDGE_VERSION > 0);

fn check_engine_bridge_version(version: u32) -> Result<(), String> {
    if version == ENGINE_BRIDGE_VERSION {
        return Ok(());
    }
    Err(format!(
        "voice engine bridge version mismatch: host sent {version}, native addon expects {ENGINE_BRIDGE_VERSION}"
    ))
}

#[napi]
pub fn get_engine_bridge_version() -> u32 {
    ENGINE_BRIDGE_VERSION
}

#[napi]
pub fn assert_engine_bridge_version(version: u32) -> napi::Result<()> {
    check_engine_bridge_version(version).map_err(napi::Error::from_reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_version_passes() {
        assert!(check_engine_bridge_version(ENGINE_BRIDGE_VERSION).is_ok());
        assert!(assert_engine_bridge_version(ENGINE_BRIDGE_VERSION).is_ok());
    }

    #[test]
    fn mismatched_version_fails_with_both_versions_in_message() {
        let error = check_engine_bridge_version(ENGINE_BRIDGE_VERSION + 1).unwrap_err();
        assert!(error.contains("voice engine bridge version mismatch"));
        assert!(error.contains(&(ENGINE_BRIDGE_VERSION + 1).to_string()));
        assert!(error.contains(&ENGINE_BRIDGE_VERSION.to_string()));
    }

    #[test]
    fn zero_version_fails() {
        assert!(check_engine_bridge_version(0).is_err());
    }

    #[test]
    fn mismatch_surfaces_as_napi_error() {
        let error = assert_engine_bridge_version(ENGINE_BRIDGE_VERSION - 1).unwrap_err();
        assert!(
            error
                .reason
                .contains("voice engine bridge version mismatch")
        );
    }

    #[test]
    fn exported_getter_reports_the_constant() {
        assert_eq!(get_engine_bridge_version(), ENGINE_BRIDGE_VERSION);
    }
}
