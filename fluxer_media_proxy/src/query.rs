// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::external_path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Target<'a> {
    pub path: &'a str,
    pub query: &'a str,
}

pub fn split_target(target: &str) -> Target<'_> {
    match target.find('?') {
        Some(q) => Target {
            path: &target[..q],
            query: &target[q + 1..],
        },
        None => Target {
            path: target,
            query: "",
        },
    }
}

#[derive(Clone, Debug, Default)]
pub struct Query {
    pairs: Vec<(String, String)>,
}

impl Query {
    pub fn parse(raw: &str) -> Self {
        let mut pairs = Vec::new();
        for field in raw.split('&') {
            if field.is_empty() {
                continue;
            }
            let eq = field.find('=').unwrap_or(field.len());
            let key = external_path::percent_decode_string(&field[..eq], true);
            let value = if eq < field.len() {
                external_path::percent_decode_string(&field[eq + 1..], true)
            } else {
                String::new()
            };
            pairs.push((key, value));
        }
        Self { pairs }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find_map(|(k, v)| (k == key).then_some(v.as_str()))
    }

    pub fn bool_value(&self, key: &str, default_value: bool) -> bool {
        self.get(key)
            .map(|raw| raw.eq_ignore_ascii_case("true") || raw == "1")
            .unwrap_or(default_value)
    }
}
