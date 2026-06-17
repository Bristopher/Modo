// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::config::AppProxyConfig;
use fluxer_common::geoip::{GeoipConfig, GeoipLookup, GeoipResolver};

pub fn resolver_from_app_config(config: &AppProxyConfig) -> GeoipResolver {
    GeoipResolver::from_config(&GeoipConfig {
        geoip_source: config.geoip_source.clone(),
        geoip_s3_config: config.geoip_s3_config.clone(),
        trust_client_ip_header: config.trust_client_ip_header,
        client_ip_header_name: config.client_ip_header_name.clone(),
    })
}

pub fn build_geoip_response(lookup: GeoipLookup) -> serde_json::Value {
    serde_json::json!({
        "countryCode": lookup.country_code,
        "regionCode": lookup.region_code,
        "latitude": lookup.latitude.map(|v| v.to_string()),
        "longitude": lookup.longitude.map(|v| v.to_string()),
        "ageRestrictedGeos": [
            {"countryCode": "GB", "regionCode": null},
            {"countryCode": "BR", "regionCode": null}
        ],
        "ageBlockedGeos": [
            {"countryCode": "US", "regionCode": "MS"}
        ]
    })
}
