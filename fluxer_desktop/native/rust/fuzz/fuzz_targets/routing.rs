#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use arbitrary::Arbitrary;
use fluxer_desktop_native::linux_audio::routing::{
    MEDIA_CLASS_PLAYBACK_STREAM, RoutingRule, SelfIdentity, should_route_node,
};
use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;

#[derive(Arbitrary, Debug)]
struct Input {
    id: u32,
    sink_id: u32,
    default_name: String,
    default_id: String,
    include_name: String,
    app_name: String,
    process_id: String,
    binary: String,
    has_device_id: bool,
    media_class_is_playback: bool,
}

fuzz_target!(|input: Input| {
    let mut props = BTreeMap::new();
    props.insert(
        "media.class".to_owned(),
        if input.media_class_is_playback {
            MEDIA_CLASS_PLAYBACK_STREAM.to_owned()
        } else {
            "Audio/Source".to_owned()
        },
    );
    props.insert("application.name".to_owned(), input.app_name);
    props.insert("application.process.id".to_owned(), input.process_id);
    props.insert("application.process.binary".to_owned(), input.binary);
    props.insert("target.object".to_owned(), input.default_name.clone());
    if input.has_device_id {
        props.insert("device.id".to_owned(), "5".to_owned());
    }
    let mut include = BTreeMap::new();
    include.insert("application.name".to_owned(), input.include_name);
    let rule = RoutingRule {
        include_when: vec![include],
        skip_hardware_devices: true,
        ..RoutingRule::default()
    };
    let self_identity = SelfIdentity::default();
    let _ = should_route_node(
        input.id,
        &props,
        &rule,
        &input.default_name,
        &input.default_id,
        input.sink_id,
        &self_identity,
    );
});
