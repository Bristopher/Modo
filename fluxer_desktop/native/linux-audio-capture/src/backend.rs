#![allow(dead_code)]

// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::routing::{PropMap, RoutingRule, SelfIdentity};

#[derive(Default, Clone)]
pub struct RoutingGraphSnapshot {
    pub backend: String,
    pub nodes: Vec<RoutingGraphNode>,
    pub ports: Vec<RoutingGraphPort>,
    pub owned_links: Vec<RoutingGraphLink>,
}

#[derive(Default, Clone)]
pub struct RoutingGraphNode {
    pub id: u32,
    pub props: PropMap,
}

#[derive(Default, Clone)]
pub struct RoutingGraphPort {
    pub id: u32,
    pub node_id: u32,
    pub direction: String,
    pub channel: String,
    pub props: PropMap,
}

#[derive(Default, Clone, Copy)]
pub struct RoutingGraphLink {
    pub output_node_id: u32,
    pub output_port_id: u32,
    pub input_node_id: u32,
    pub input_port_id: u32,
}

pub trait CaptureBridge: Send + Sync {
    fn inventory(&self) -> Vec<PropMap>;
    fn apply(&self, rule: RoutingRule) -> bool;
    fn release(&self);
    fn populate_self_identity(&self, identity: SelfIdentity);
    fn backend_name(&self) -> &'static str;

    fn routing_graph(&self) -> RoutingGraphSnapshot {
        RoutingGraphSnapshot {
            backend: self.backend_name().to_string(),
            ..Default::default()
        }
    }
}

pub trait DirectCapture: Send + Sync {
    fn start(&self, rule: RoutingRule) -> bool;

    fn set_rule(&self, rule: RoutingRule) -> bool;
    fn read(&self) -> Option<CapturedFrame>;
    fn stop(&self);
    fn populate_self_identity(&self, identity: SelfIdentity);

    fn set_screen_audio_sink(
        &self,
        _sink: std::sync::Arc<fluxer_screen_frame_bus::NativeScreenFrameSinkHandleRef>,
    ) {
    }

    fn clear_screen_audio_sink(&self) {}

    fn routing_graph(&self) -> RoutingGraphSnapshot {
        RoutingGraphSnapshot::default()
    }

    fn last_push_ns_arc(&self) -> Option<std::sync::Arc<std::sync::atomic::AtomicU64>> {
        None
    }
}

pub struct CapturedFrame {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u32,
    pub timestamp_us: i64,
}
