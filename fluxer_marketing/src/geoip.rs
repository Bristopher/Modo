// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::config::MarketingConfig;
use axum::http::HeaderMap;
use fluxer_common::geoip::{GeoipConfig, GeoipResolver};

pub fn resolver_from_marketing_config(config: &MarketingConfig) -> GeoipResolver {
    GeoipResolver::from_config(&GeoipConfig {
        geoip_source: config.geoip_source.clone(),
        geoip_s3_config: config.geoip_s3_config.clone(),
        trust_client_ip_header: config.trust_client_ip_header,
        client_ip_header_name: config.client_ip_header_name.clone(),
    })
}

pub fn country_code(resolver: &GeoipResolver, headers: &HeaderMap) -> String {
    resolver.country_code(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn no_reader_returns_default() {
        let resolver = GeoipResolver::from_config(&GeoipConfig {
            geoip_source: fluxer_common::config::GeoipSourceConfig::Filesystem {
                maxmind_db_path: None,
            },
            geoip_s3_config: None,
            trust_client_ip_header: true,
            client_ip_header_name: "x-forwarded-for".to_owned(),
        });
        let mut headers = HeaderMap::new();
        headers.insert("cf-ipcountry", HeaderValue::from_static("SE"));
        headers.insert("x-vercel-ip-country", HeaderValue::from_static("SE"));
        assert_eq!(resolver.country_code(&headers), "US");
    }
}
