// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::{Arc, Mutex};

use pipewire as pw;

use crate::routing::{
    MEDIA_CLASS_PLAYBACK_STREAM, PropMap, RoutingRule, SelfIdentity, matches_any, should_route_node,
};

use super::common::{
    InventorySnapshot, LinkKey, MEDIA_CLASS_CAPTURE_STREAM, MetadataWatch, OwnedLink,
    PIN_TARGET_METADATA_KEY, PIN_TARGET_METADATA_TYPE, destroy_owned_links, pick_node_ports,
    pick_source_output_ports, sync_owned_links,
};

#[derive(Default)]
pub(crate) struct RoutingState {
    pub(crate) identity: SelfIdentity,
    pub(crate) active_rule: Option<RoutingRule>,
    pub(crate) default_sink_name: String,
    pub(crate) sink_global_id: u32,
    pub(crate) pinned_capture_nodes: HashSet<u32>,
    pub(crate) sink_input_fl: Option<u32>,
    pub(crate) sink_input_fr: Option<u32>,
}

pub(crate) fn refresh_sink_input_ports(
    inventory: &Arc<Mutex<InventorySnapshot>>,
    state: &std::rc::Rc<std::cell::RefCell<RoutingState>>,
) {
    let sink_id = state.borrow().sink_global_id;
    if sink_id == 0 {
        return;
    }
    assert!(sink_id != 0);
    let Ok(snap) = inventory.lock() else {
        return;
    };
    let ports = pick_node_ports(sink_id, "in", &snap.ports);
    drop(snap);
    if let Some((fl, fr)) = ports {
        assert!(fl != 0);
        assert!(fr != 0);
        let mut st = state.borrow_mut();
        st.sink_input_fl = Some(fl);
        st.sink_input_fr = Some(fr);
    }
}

pub(crate) fn default_sink_target_id(
    nodes: &HashMap<u32, PropMap>,
    default_sink_name: &str,
) -> String {
    if default_sink_name.is_empty() {
        return String::new();
    }
    nodes
        .values()
        .find(|props| {
            props
                .get("node.name")
                .is_some_and(|name| name == default_sink_name)
        })
        .and_then(|props| props.get("object.serial").cloned())
        .unwrap_or_default()
}

pub(crate) fn matching_pinned_capture_nodes(
    nodes: &HashMap<u32, PropMap>,
    rule: &RoutingRule,
    sink_global_id: u32,
) -> HashSet<u32> {
    if rule.pin_target_for.is_empty() {
        return HashSet::new();
    }
    nodes
        .iter()
        .filter_map(|(node_id, props)| {
            if *node_id == sink_global_id {
                return None;
            }
            let is_capture_stream = props
                .get("media.class")
                .is_some_and(|class| class == MEDIA_CLASS_CAPTURE_STREAM);
            if is_capture_stream && matches_any(props, &rule.pin_target_for) {
                Some(*node_id)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn sync_pinned_capture_targets(
    nodes: &HashMap<u32, PropMap>,
    rule: &RoutingRule,
    sink_global_id: u32,
    sink_node_name: &str,
    state: &std::rc::Rc<std::cell::RefCell<RoutingState>>,
    metadata_watchers: &std::rc::Rc<std::cell::RefCell<Vec<MetadataWatch>>>,
) {
    let desired = matching_pinned_capture_nodes(nodes, rule, sink_global_id);
    let previous = state.borrow().pinned_capture_nodes.clone();
    for node_id in previous.difference(&desired) {
        set_pinned_capture_target(metadata_watchers, *node_id, None);
    }
    for node_id in &desired {
        set_pinned_capture_target(metadata_watchers, *node_id, Some(sink_node_name));
    }
    state.borrow_mut().pinned_capture_nodes = desired;
}

pub(crate) fn clear_pinned_capture_targets(
    state: &std::rc::Rc<std::cell::RefCell<RoutingState>>,
    metadata_watchers: &std::rc::Rc<std::cell::RefCell<Vec<MetadataWatch>>>,
) {
    let pinned = mem::take(&mut state.borrow_mut().pinned_capture_nodes);
    for node_id in pinned {
        set_pinned_capture_target(metadata_watchers, node_id, None);
    }
}

pub(crate) fn set_pinned_capture_target(
    metadata_watchers: &std::rc::Rc<std::cell::RefCell<Vec<MetadataWatch>>>,
    node_id: u32,
    target: Option<&str>,
) {
    assert!(node_id != 0);
    let watchers = metadata_watchers.borrow();
    for watcher in watchers.iter().filter(|watcher| watcher.is_default) {
        watcher.metadata.set_property(
            node_id,
            PIN_TARGET_METADATA_KEY,
            target.map(|_| PIN_TARGET_METADATA_TYPE),
            target,
        );
    }
}

struct RoutingResolved {
    rule: RoutingRule,
    sink_id: u32,
    sink_in_fl: u32,
    sink_in_fr: u32,
    identity: SelfIdentity,
    default_sink_name: String,
}

fn resolve_routing_inputs(
    state: &std::rc::Rc<std::cell::RefCell<RoutingState>>,
) -> Option<RoutingResolved> {
    let st = state.borrow();
    let rule = st.active_rule.clone()?;
    let sink_id = st.sink_global_id;
    if sink_id == 0 {
        return None;
    }
    let (sink_in_fl, sink_in_fr) = (st.sink_input_fl?, st.sink_input_fr?);
    Some(RoutingResolved {
        rule,
        sink_id,
        sink_in_fl,
        sink_in_fr,
        identity: st.identity.clone(),
        default_sink_name: st.default_sink_name.clone(),
    })
}

fn build_desired_links(
    nodes: &HashMap<u32, PropMap>,
    ports: &HashMap<u32, super::common::PortRecord>,
    resolved: &RoutingResolved,
    default_sink_target_id: &str,
) -> Vec<LinkKey> {
    let mut desired_links = Vec::new();
    for (node_id, props) in nodes {
        if *node_id == resolved.sink_id {
            continue;
        }
        if !should_route_node(
            *node_id,
            props,
            &resolved.rule,
            &resolved.default_sink_name,
            default_sink_target_id,
            resolved.sink_id,
            &resolved.identity,
        ) {
            continue;
        }
        let Some((src_l, src_r)) = pick_source_output_ports(*node_id, ports) else {
            continue;
        };
        desired_links.push(LinkKey::new(
            *node_id,
            src_l,
            resolved.sink_id,
            resolved.sink_in_fl,
        ));
        desired_links.push(LinkKey::new(
            *node_id,
            src_r,
            resolved.sink_id,
            resolved.sink_in_fr,
        ));
    }
    desired_links
}

pub(crate) fn recompute_routing(
    inventory: &Arc<Mutex<InventorySnapshot>>,
    state: &std::rc::Rc<std::cell::RefCell<RoutingState>>,
    core: &pw::core::CoreRc,
    owned_links: &std::rc::Rc<std::cell::RefCell<Vec<OwnedLink>>>,
    owned_link_snapshot: &Arc<Mutex<Vec<LinkKey>>>,
    metadata_watchers: &std::rc::Rc<std::cell::RefCell<Vec<MetadataWatch>>>,
    sink_node_name: &str,
) {
    let Some(resolved) = resolve_routing_inputs(state) else {
        clear_pinned_capture_targets(state, metadata_watchers);
        destroy_owned_links(core, owned_links, owned_link_snapshot);
        return;
    };

    let Ok(snap) = inventory.lock() else {
        return;
    };
    let nodes = snap.enriched_nodes();
    let ports = snap.ports.clone();
    drop(snap);
    let default_sink_target_id = default_sink_target_id(&nodes, &resolved.default_sink_name);

    sync_pinned_capture_targets(
        &nodes,
        &resolved.rule,
        resolved.sink_id,
        sink_node_name,
        state,
        metadata_watchers,
    );

    let desired_links = build_desired_links(&nodes, &ports, &resolved, &default_sink_target_id);
    sync_owned_links(core, owned_links, owned_link_snapshot, desired_links);
}

#[derive(Default)]
pub(crate) struct DirectRoutingState {
    pub(crate) identity: SelfIdentity,
    pub(crate) active_rule: Option<RoutingRule>,
    pub(crate) default_sink_name: String,
    pub(crate) sink_global_id: u32,
    pub(crate) sink_input_fl: Option<u32>,
    pub(crate) sink_input_fr: Option<u32>,
}

pub(crate) fn refresh_direct_sink_input_ports(
    inventory: &Arc<Mutex<InventorySnapshot>>,
    state: &std::rc::Rc<std::cell::RefCell<DirectRoutingState>>,
) {
    let sink_id = state.borrow().sink_global_id;
    if sink_id == 0 {
        return;
    }
    let Ok(snap) = inventory.lock() else {
        return;
    };
    let ports = pick_node_ports(sink_id, "in", &snap.ports);
    drop(snap);
    if let Some((fl, fr)) = ports {
        let mut st = state.borrow_mut();
        st.sink_input_fl = Some(fl);
        st.sink_input_fr = Some(fr);
    }
}

struct DirectResolved {
    rule: RoutingRule,
    sink_id: u32,
    sink_in_fl: u32,
    sink_in_fr: u32,
    identity: SelfIdentity,
    default_sink_name: String,
}

fn resolve_direct_inputs(
    state: &std::rc::Rc<std::cell::RefCell<DirectRoutingState>>,
) -> Option<DirectResolved> {
    let st = state.borrow();
    let rule = st.active_rule.clone()?;
    let sink_id = st.sink_global_id;
    let (sink_in_fl, sink_in_fr) = (st.sink_input_fl?, st.sink_input_fr?);
    Some(DirectResolved {
        rule,
        sink_id,
        sink_in_fl,
        sink_in_fr,
        identity: st.identity.clone(),
        default_sink_name: st.default_sink_name.clone(),
    })
}

fn build_direct_desired_links(
    nodes: &HashMap<u32, PropMap>,
    ports: &HashMap<u32, super::common::PortRecord>,
    resolved: &DirectResolved,
    default_sink_target_id: &str,
) -> Vec<LinkKey> {
    let mut desired_links = Vec::new();
    for (node_id, props) in nodes {
        if *node_id == resolved.sink_id {
            continue;
        }
        if !should_route_node(
            *node_id,
            props,
            &resolved.rule,
            &resolved.default_sink_name,
            default_sink_target_id,
            resolved.sink_id,
            &resolved.identity,
        ) {
            continue;
        }
        let Some((src_l, src_r)) = pick_source_output_ports(*node_id, ports) else {
            continue;
        };
        desired_links.push(LinkKey::new(
            *node_id,
            src_l,
            resolved.sink_id,
            resolved.sink_in_fl,
        ));
        desired_links.push(LinkKey::new(
            *node_id,
            src_r,
            resolved.sink_id,
            resolved.sink_in_fr,
        ));
    }
    desired_links
}

pub(crate) fn recompute_direct_links(
    core: &pw::core::CoreRc,
    inventory: &Arc<Mutex<InventorySnapshot>>,
    state: &std::rc::Rc<std::cell::RefCell<DirectRoutingState>>,
    owned_links: &std::rc::Rc<std::cell::RefCell<Vec<OwnedLink>>>,
    owned_link_snapshot: &Arc<Mutex<Vec<LinkKey>>>,
) -> bool {
    let Some(resolved) = resolve_direct_inputs(state) else {
        return false;
    };
    let Ok(snap) = inventory.lock() else {
        return false;
    };
    let nodes = snap.enriched_nodes();
    let ports = snap.ports.clone();
    drop(snap);
    let default_sink_target_id = default_sink_target_id(&nodes, &resolved.default_sink_name);
    let desired_links =
        build_direct_desired_links(&nodes, &ports, &resolved, &default_sink_target_id);
    sync_owned_links(core, owned_links, owned_link_snapshot, desired_links);
    true
}

const _: () = {
    let _ = MEDIA_CLASS_PLAYBACK_STREAM;
};
