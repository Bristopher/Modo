// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::config::AppProxyConfig;
use crate::discovery_cache::DiscoveryCache;
use crate::invite_meta::InviteMetaResolver;
use fluxer_common::geoip::GeoipResolver;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppProxyConfig>,
    pub http_client: reqwest::Client,
    pub discovery_cache: Arc<DiscoveryCache>,
    pub geoip: Arc<GeoipResolver>,
    pub invite_meta: Option<Arc<InviteMetaResolver>>,
    pub index_html: Option<Arc<str>>,
}
