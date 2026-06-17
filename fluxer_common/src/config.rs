// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{env, path::Path};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeoipSourceConfig {
    Filesystem {
        maxmind_db_path: Option<String>,
    },
    S3 {
        maxmind_db_path: String,
        maxmind_asn_db_path: Option<String>,
        s3_bucket: String,
        s3_key: String,
        s3_asn_key: Option<String>,
    },
}

impl GeoipSourceConfig {
    pub fn maxmind_db_path(&self) -> Option<String> {
        match self {
            Self::Filesystem { maxmind_db_path } => maxmind_db_path.clone(),
            Self::S3 {
                maxmind_db_path, ..
            } => Some(maxmind_db_path.clone()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoipS3Config {
    pub endpoint: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

pub fn read_geoip_s3_config_from_env(source: &GeoipSourceConfig) -> Option<GeoipS3Config> {
    read_geoip_s3_config(source, |name| env::var(name).ok())
}

fn read_geoip_s3_config<F>(source: &GeoipSourceConfig, mut read_var: F) -> Option<GeoipS3Config>
where
    F: FnMut(&str) -> Option<String>,
{
    match source {
        GeoipSourceConfig::S3 { .. } => Some(GeoipS3Config {
            endpoint: read_var("FLUXER_S3_ENDPOINT").unwrap_or_default(),
            region: read_var("FLUXER_S3_REGION").unwrap_or_default(),
            access_key_id: read_var("FLUXER_S3_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: read_var("FLUXER_S3_SECRET_ACCESS_KEY").unwrap_or_default(),
        }),
        GeoipSourceConfig::Filesystem { .. } => None,
    }
}

pub fn parse_geoip_source_config(raw_value: &str, service_name: &str) -> GeoipSourceConfig {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return GeoipSourceConfig::Filesystem {
            maxmind_db_path: None,
        };
    }
    if !trimmed.starts_with("s3://") {
        return GeoipSourceConfig::Filesystem {
            maxmind_db_path: Some(trimmed.to_owned()),
        };
    }
    parse_geoip_s3_source_config(trimmed, service_name)
}

fn parse_geoip_s3_source_config(raw_value: &str, service_name: &str) -> GeoipSourceConfig {
    let url = reqwest::Url::parse(raw_value)
        .unwrap_or_else(|err| panic!("invalid GeoIP S3 URL {}: {}", raw_value, err));
    let s3_bucket = url.host_str().unwrap_or("").to_owned();
    if s3_bucket.is_empty() {
        panic!("invalid GeoIP S3 URL (missing bucket): {raw_value}");
    }
    let s3_key = percent_decode(url.path().trim_start_matches('/'));
    if s3_key.is_empty() {
        panic!("invalid GeoIP S3 URL (missing object key): {raw_value}");
    }
    let maxmind_db_path =
        geoip_runtime_path(&resolve_geoip_download_path(&url, raw_value), service_name);
    let s3_asn_key = url
        .query_pairs()
        .find(|(key, _)| key == "asn_key")
        .map(|(_, value)| value.into_owned());
    let maxmind_asn_db_path = s3_asn_key.as_ref().map(|asn_key| {
        let configured_path = url
            .query_pairs()
            .find(|(key, _)| key == "asn_download_path")
            .map(|(_, value)| require_absolute_path(value.as_ref(), "asn_download_path", raw_value))
            .unwrap_or_else(|| {
                let directory = Path::new(&maxmind_db_path)
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_default();
                directory
                    .join(Path::new(asn_key).file_name().unwrap_or_default())
                    .to_string_lossy()
                    .into_owned()
            });
        geoip_runtime_path(&configured_path, service_name)
    });
    GeoipSourceConfig::S3 {
        maxmind_db_path,
        maxmind_asn_db_path,
        s3_bucket,
        s3_key,
        s3_asn_key,
    }
}

fn resolve_geoip_download_path(url: &reqwest::Url, raw_value: &str) -> String {
    let Some(download_path) = url
        .query_pairs()
        .find(|(key, _)| key == "download_path")
        .map(|(_, value)| value.into_owned())
    else {
        panic!("invalid GeoIP S3 URL (missing query parameter \"download_path\"): {raw_value}");
    };
    require_absolute_path(&download_path, "download_path", raw_value)
}

fn require_absolute_path(value: &str, param: &str, raw_value: &str) -> String {
    if !Path::new(value).is_absolute() {
        panic!("GeoIP S3 URL query parameter \"{param}\" must be an absolute path: {raw_value}");
    }
    value.to_owned()
}

fn geoip_runtime_path(configured_path: &str, service_name: &str) -> String {
    let basename = Path::new(configured_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("GeoLite2-City.mmdb");
    Path::new("/tmp/fluxer/geoip")
        .join(service_name)
        .join(basename)
        .to_string_lossy()
        .into_owned()
}

fn percent_decode(value: &str) -> String {
    urlencoding::decode(value)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| value.to_owned())
}

pub fn read_env(name: &str, fallback: &str) -> String {
    env::var(name).unwrap_or_else(|_| fallback.to_owned())
}

pub fn read_env_preferred(names: &[&str], fallback: &str) -> String {
    names
        .iter()
        .find_map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()))
        .unwrap_or_else(|| fallback.to_owned())
}

pub fn read_first_env(names: &[&str], fallback: &str) -> String {
    names
        .iter()
        .find_map(|name| env::var(name).ok())
        .unwrap_or_else(|| fallback.to_owned())
}

pub fn read_bool_env(names: &[&str], fallback: bool) -> bool {
    let Some(value) = names.iter().find_map(|name| env::var(name).ok()) else {
        return fallback;
    };
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

pub fn normalize_base_path(value: &str) -> String {
    let trimmed = value.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

pub fn trim_trailing_slash(value: &str) -> String {
    value.trim_end_matches('/').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_filesystem_geoip_source() {
        let source = parse_geoip_source_config("/data/GeoLite2-City.mmdb", "test");
        assert_eq!(
            source,
            GeoipSourceConfig::Filesystem {
                maxmind_db_path: Some("/data/GeoLite2-City.mmdb".to_owned()),
            }
        );
    }

    #[test]
    fn parses_empty_geoip_source() {
        let source = parse_geoip_source_config("", "test");
        assert_eq!(
            source,
            GeoipSourceConfig::Filesystem {
                maxmind_db_path: None,
            }
        );
    }

    #[test]
    fn parses_s3_geoip_source() {
        let source = parse_geoip_source_config(
            "s3://geoip/GeoLite2-City.mmdb?download_path=/tmp/city.mmdb&asn_key=GeoLite2-ASN.mmdb",
            "test_svc",
        );
        assert_eq!(
            source,
            GeoipSourceConfig::S3 {
                maxmind_db_path: "/tmp/fluxer/geoip/test_svc/city.mmdb".to_owned(),
                maxmind_asn_db_path: Some(
                    "/tmp/fluxer/geoip/test_svc/GeoLite2-ASN.mmdb".to_owned()
                ),
                s3_bucket: "geoip".to_owned(),
                s3_key: "GeoLite2-City.mmdb".to_owned(),
                s3_asn_key: Some("GeoLite2-ASN.mmdb".to_owned()),
            }
        );
    }

    #[test]
    fn reads_geoip_s3_config_only_for_s3_source() {
        let source = parse_geoip_source_config(
            "s3://geoip/GeoLite2-City.mmdb?download_path=/tmp/city.mmdb",
            "test_svc",
        );
        let config = read_geoip_s3_config(&source, |name| {
            Some(
                match name {
                    "FLUXER_S3_ENDPOINT" => "https://s3.example.test",
                    "FLUXER_S3_REGION" => "ewr1",
                    "FLUXER_S3_ACCESS_KEY_ID" => "access",
                    "FLUXER_S3_SECRET_ACCESS_KEY" => "secret",
                    _ => "",
                }
                .to_owned(),
            )
        })
        .expect("s3 source should read s3 config");

        assert_eq!(
            config,
            GeoipS3Config {
                endpoint: "https://s3.example.test".to_owned(),
                region: "ewr1".to_owned(),
                access_key_id: "access".to_owned(),
                secret_access_key: "secret".to_owned(),
            }
        );

        let filesystem_source = parse_geoip_source_config("/tmp/city.mmdb", "test_svc");
        assert!(read_geoip_s3_config(&filesystem_source, |_| None).is_none());
    }

    #[test]
    fn normalize_base_path_strips_slashes() {
        assert_eq!(normalize_base_path("/foo/bar/"), "/foo/bar");
        assert_eq!(normalize_base_path("foo"), "/foo");
        assert_eq!(normalize_base_path(""), "");
        assert_eq!(normalize_base_path("/"), "");
    }

    #[test]
    fn trim_trailing_slash_works() {
        assert_eq!(
            trim_trailing_slash("https://example.com/"),
            "https://example.com"
        );
        assert_eq!(trim_trailing_slash(""), "");
    }
}
