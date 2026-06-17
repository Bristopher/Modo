// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Attribute {
    name: String,
    value: Option<String>,
}

pub fn normalize_body(content_type: Option<&str>, body: &str) -> String {
    let content_type = content_type.unwrap_or("").to_ascii_lowercase();
    if content_type.contains("css") {
        return normalize_css_body(body);
    }
    if content_type.contains("html") {
        return normalize_html(body);
    }
    if content_type.contains("json")
        && let Ok(value) = serde_json::from_str::<Value>(body)
    {
        return serde_json::to_string(&value).unwrap_or_else(|_| normalize_text(body));
    }
    normalize_text(body)
}

fn normalize_css_body(body: &str) -> String {
    if body.trim().is_empty() || body.contains("not available") {
        "__CSS_EMPTY__".to_owned()
    } else {
        "__CSS_OK__".to_owned()
    }
}

pub fn normalize_html(input: &str) -> String {
    let input = normalize_text(input);
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative_start) = input[cursor..].find('<') {
        let start = cursor + relative_start;
        output.push_str(&input[cursor..start]);
        let Some(end) = find_tag_end(&input, start) else {
            output.push_str(&input[start..]);
            return finish_html(output);
        };
        let raw_tag = &input[start..=end];
        output.push_str(&normalize_tag(raw_tag));
        cursor = end + 1;
    }
    output.push_str(&input[cursor..]);
    finish_html(output)
}

pub fn normalize_text(input: &str) -> String {
    let mut value = input.replace("\r\n", "\n").replace('\r', "\n");
    value = normalize_host_ports(&value);
    value = strip_static_asset_queries(&value);
    value = strip_htmx_script_tags(&value);
    value = strip_rust_runtime_script_blocks(&value);
    value = normalize_intentional_rust_improvements(&value);
    value = replace_query_parameter_values(&value, "state", "__OAUTH_STATE__");
    value = replace_query_parameter_values(&value, "_csrf", "__CSRF_TOKEN__");
    value = replace_script_csrf_values(&value);
    value = normalize_cookie_tokens(&value);
    value.trim().to_owned()
}

pub fn normalize_header_value(name: &str, value: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "content-type" => normalize_content_type(value),
        "location" => normalize_text(value),
        "cookie" | "set-cookie" => normalize_cookie_header(value),
        _ => normalize_text(value),
    }
}

pub fn normalize_content_type(value: &str) -> String {
    value
        .split(';')
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn normalize_cookie_header(value: &str) -> String {
    normalize_cookie_tokens(value)
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn replace_script_csrf_values(input: &str) -> String {
    let pattern = "var csrf=\"";
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative_start) = input[cursor..].find(pattern) {
        let start = cursor + relative_start;
        let token_start = start + pattern.len();
        output.push_str(&input[cursor..token_start]);
        let Some(relative_end) = input[token_start..].find('"') else {
            cursor = token_start;
            break;
        };
        let token_end = token_start + relative_end;
        let token = &input[token_start..token_end];
        if token.len() == 64 && token.chars().all(|ch| ch.is_ascii_hexdigit()) {
            output.push_str("__CSRF_TOKEN__");
        } else {
            output.push_str(token);
        }
        cursor = token_end;
    }
    output.push_str(&input[cursor..]);
    output
}

fn finish_html(output: String) -> String {
    let output = remove_empty_flash_container(&output);
    let output = decode_html_entities(&output);
    remove_between_tag_whitespace(&output).trim().to_owned()
}

fn remove_empty_flash_container(input: &str) -> String {
    input
        .replace(
            r#"<div class="empty:hidden" id="flash-container"></div>"#,
            "",
        )
        .replace(
            r#"<div id="flash-container" class="empty:hidden"></div>"#,
            "",
        )
}

fn find_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (offset, ch) in input[start + 1..].char_indices() {
        match (quote, ch) {
            (Some(active), current) if current == active => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => return Some(start + 1 + offset),
            _ => {}
        }
    }
    None
}

fn normalize_tag(raw: &str) -> String {
    if raw.starts_with("<!--") {
        return String::new();
    }
    if raw.len() < 3 {
        return raw.to_owned();
    }
    let mut inner = raw[1..raw.len() - 1].trim();
    let self_closing = inner.ends_with('/');
    if self_closing {
        inner = inner[..inner.len() - 1].trim_end();
    }
    if inner.eq_ignore_ascii_case("!doctype html") {
        return String::new();
    }
    if inner.starts_with('!') || inner.starts_with('?') {
        return format!("<{}>", collapse_ascii_whitespace(inner));
    }
    if let Some(rest) = inner.strip_prefix('/') {
        return format!("</{}>", rest.trim().to_ascii_lowercase());
    }
    let (tag_name, rest) = split_tag_name(inner);
    let tag_name = tag_name.to_ascii_lowercase();
    let mut attributes = parse_attributes(rest);
    normalize_csrf_input_value(&tag_name, &mut attributes);
    let tag_attributes = attributes.clone();
    attributes.retain(|attribute| {
        !is_ignored_rust_runtime_attribute(&tag_name, &tag_attributes, attribute)
    });
    for attribute in &mut attributes {
        if let Some(value) = attribute.value.take() {
            attribute.value = Some(normalize_text(&decode_html_entities(&value)));
        }
        attribute.name = attribute.name.to_ascii_lowercase();
        if attribute.name == "class"
            && let Some(ref mut value) = attribute.value
        {
            let mut tokens: Vec<&str> = value.split_whitespace().collect();
            tokens.retain(|token| *token != "font-normal");
            tokens.sort();
            *value = tokens.join(" ");
        }
        if attribute.value.as_deref() == Some("") && is_boolean_attribute(&attribute.name) {
            attribute.value = None;
        }
    }
    attributes.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.value.cmp(&right.value))
    });
    let mut normalized = format!("<{tag_name}");
    for attribute in attributes {
        normalized.push(' ');
        normalized.push_str(&attribute.name);
        if let Some(value) = attribute.value {
            normalized.push_str("=\"");
            normalized.push_str(&escape_attr(&value));
            normalized.push('"');
        }
    }
    normalized.push('>');
    if self_closing && !is_void_element(&tag_name) {
        normalized.push_str("</");
        normalized.push_str(&tag_name);
        normalized.push('>');
    }
    normalized
}

fn split_tag_name(inner: &str) -> (&str, &str) {
    let split_at = inner
        .find(|ch: char| ch.is_ascii_whitespace())
        .unwrap_or(inner.len());
    let name = &inner[..split_at];
    let rest = inner[split_at..].trim_start();
    (name, rest)
}

fn parse_attributes(input: &str) -> Vec<Attribute> {
    let bytes = input.as_bytes();
    let mut attributes = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'='
        {
            cursor += 1;
        }
        if name_start == cursor {
            cursor += 1;
            continue;
        }
        let name = input[name_start..cursor].to_owned();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            attributes.push(Attribute { name, value: None });
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            attributes.push(Attribute {
                name,
                value: Some(String::new()),
            });
            break;
        }
        let value = if bytes[cursor] == b'"' || bytes[cursor] == b'\'' {
            let quote = bytes[cursor];
            cursor += 1;
            let value_start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                cursor += 1;
            }
            let value = input[value_start..cursor].to_owned();
            if cursor < bytes.len() {
                cursor += 1;
            }
            value
        } else {
            let value_start = cursor;
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            input[value_start..cursor].to_owned()
        };
        attributes.push(Attribute {
            name,
            value: Some(value),
        });
    }
    attributes
}

fn normalize_csrf_input_value(tag_name: &str, attributes: &mut [Attribute]) {
    if tag_name != "input" {
        return;
    }
    let is_csrf = attributes.iter().any(|attribute| {
        attribute.name.eq_ignore_ascii_case("name") && attribute.value.as_deref() == Some("_csrf")
    });
    if !is_csrf {
        return;
    }
    for attribute in attributes {
        if attribute.name.eq_ignore_ascii_case("value") {
            attribute.value = Some("__CSRF_TOKEN__".to_owned());
        }
    }
}

fn is_ignored_rust_runtime_attribute(
    tag_name: &str,
    tag_attributes: &[Attribute],
    attribute: &Attribute,
) -> bool {
    let name = attribute.name.to_ascii_lowercase();
    if name.starts_with("hx-") {
        return true;
    }
    if matches!(
        name.as_str(),
        "popovertarget" | "popovertargetaction" | "popover"
    ) {
        return true;
    }
    let is_drawer_panel = tag_name == "aside"
        && tag_attributes
            .iter()
            .any(|attr| attr.name.eq_ignore_ascii_case("data-drawer-panel"));
    if is_drawer_panel && matches!(name.as_str(), "id" | "aria-hidden") {
        return true;
    }
    let is_drawer_body = tag_attributes
        .iter()
        .any(|attr| attr.name.eq_ignore_ascii_case("data-drawer-body"));
    if is_drawer_body && name == "id" {
        return true;
    }
    if name != "id" {
        return false;
    }
    matches!(
        attribute.value.as_deref(),
        Some(
            "users-results"
                | "guilds-results"
                | "applications-results"
                | "reports-results"
                | "user-peek"
                | "guild-peek"
                | "report-peek"
                | "guild-tab-content"
                | "user-tab-content"
                | "jobs-results"
        )
    )
}

fn is_boolean_attribute(name: &str) -> bool {
    if name.starts_with("data-") {
        return true;
    }
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "defer"
            | "disabled"
            | "hidden"
            | "loop"
            | "multiple"
            | "muted"
            | "open"
            | "readonly"
            | "required"
            | "selected"
    )
}

fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&larr;", "\u{2190}")
        .replace("&rarr;", "\u{2192}")
        .replace("&mdash;", "\u{2014}")
        .replace("&ndash;", "\u{2013}")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn collapse_ascii_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_between_tag_whitespace(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while cursor < chars.len() {
        output.push(chars[cursor]);
        if chars[cursor] == '>' {
            cursor += 1;
            let whitespace_start = cursor;
            while cursor < chars.len() && chars[cursor].is_whitespace() {
                cursor += 1;
            }
            if cursor < chars.len() && chars[cursor] == '<' {
                continue;
            }
            if cursor > whitespace_start {
                output.push(' ');
            }
            continue;
        }
        cursor += 1;
    }
    output
}

fn normalize_host_ports(input: &str) -> String {
    let value = replace_host_port(input, "127.0.0.1");
    replace_host_port(&value, "localhost")
}

fn replace_host_port(input: &str, host: &str) -> String {
    let needle = format!("{host}:");
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative) = input[cursor..].find(&needle) {
        let start = cursor + relative;
        output.push_str(&input[cursor..start]);
        output.push_str(host);
        output.push_str(":__PORT__");
        cursor = start + needle.len();
        while cursor < input.len() && input.as_bytes()[cursor].is_ascii_digit() {
            cursor += 1;
        }
    }
    output.push_str(&input[cursor..]);
    output
}

fn strip_static_asset_queries(input: &str) -> String {
    let mut value = strip_query_after_path(input, "/static/app.css");
    value = strip_query_after_path(&value, "/static/htmx.min.js");
    value
}

fn strip_htmx_script_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative_start) = input[cursor..].find("<script") {
        let start = cursor + relative_start;
        let Some(tag_end) = find_tag_end(input, start) else {
            break;
        };
        let tag = &input[start..=tag_end];
        if tag.to_ascii_lowercase().contains("/static/htmx.min.js") {
            output.push_str(&input[cursor..start]);
            let after_tag = tag_end + 1;
            if input[after_tag..].starts_with("</script>") {
                cursor = after_tag + "</script>".len();
            } else {
                cursor = after_tag;
            }
            continue;
        }
        output.push_str(&input[cursor..=tag_end]);
        cursor = tag_end + 1;
    }
    output.push_str(&input[cursor..]);
    output
}

fn strip_rust_runtime_script_blocks(input: &str) -> String {
    let mut value =
        strip_script_blocks_containing(input, "document.body.addEventListener('showFlash'");
    value = strip_script_blocks_containing(&value, "__fluxerAdminHtmxScrollPreserver");
    value = strip_script_blocks_containing(&value, "window.__adminCopyToClipboard");
    value = strip_script_blocks_containing(&value, "window.__fluxerDrawerInit");
    value
}

fn strip_script_blocks_containing(input: &str, needle: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative_start) = input[cursor..].find("<script") {
        let start = cursor + relative_start;
        let Some(tag_end) = find_tag_end(input, start) else {
            break;
        };
        let content_start = tag_end + 1;
        let Some(relative_end) = input[content_start..].find("</script>") else {
            break;
        };
        let end = content_start + relative_end + "</script>".len();
        let block = &input[start..end];
        if block.contains(needle) {
            output.push_str(&input[cursor..start]);
            cursor = end;
            continue;
        }
        output.push_str(&input[cursor..end]);
        cursor = end;
    }
    output.push_str(&input[cursor..]);
    output
}

fn normalize_intentional_rust_improvements(input: &str) -> String {
    input.replace(
        "flex flex-col gap-8 items-center",
        "flex flex-col gap-8 items-stretch",
    )
}

fn strip_query_after_path(input: &str, path: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative) = input[cursor..].find(path) {
        let start = cursor + relative;
        output.push_str(&input[cursor..start]);
        output.push_str(path);
        cursor = start + path.len();
        if input[cursor..].starts_with('?') {
            cursor += 1;
            while cursor < input.len() && !is_url_delimiter(input.as_bytes()[cursor]) {
                cursor += 1;
            }
        }
    }
    output.push_str(&input[cursor..]);
    output
}

fn replace_query_parameter_values(input: &str, name: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while cursor < input.len() {
        let remaining = &input[cursor..];
        let query_needle = format!("?{name}=");
        let amp_needle = format!("&{name}=");
        let query_pos = remaining.find(&query_needle);
        let amp_pos = remaining.find(&amp_needle);
        let Some((relative, needle_len)) =
            closest_match(query_pos, query_needle.len(), amp_pos, amp_needle.len())
        else {
            output.push_str(remaining);
            break;
        };
        let start = cursor + relative;
        output.push_str(&input[cursor..start + needle_len]);
        output.push_str(replacement);
        cursor = start + needle_len;
        while cursor < input.len() && !is_url_delimiter(input.as_bytes()[cursor]) {
            cursor += 1;
        }
    }
    output
}

fn closest_match(
    left: Option<usize>,
    left_len: usize,
    right: Option<usize>,
    right_len: usize,
) -> Option<(usize, usize)> {
    match (left, right) {
        (Some(left), Some(right)) if left <= right => Some((left, left_len)),
        (Some(_left), Some(right)) => Some((right, right_len)),
        (Some(left), None) => Some((left, left_len)),
        (None, Some(right)) => Some((right, right_len)),
        (None, None) => None,
    }
}

fn normalize_cookie_tokens(input: &str) -> String {
    let mut value = replace_prefixed_value(input, "admin_session=", "__SESSION__");
    value = replace_prefixed_value(&value, "session=", "__SESSION__");
    value = replace_prefixed_value(&value, "csrf_token=", "__CSRF_COOKIE__");
    value = replace_prefixed_value(&value, "oauth_state=", "__OAUTH_STATE__");
    replace_prefixed_value(&value, "flash=", "__FLASH__")
}

fn replace_prefixed_value(input: &str, prefix: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    while let Some(relative) = input[cursor..].find(prefix) {
        let start = cursor + relative;
        output.push_str(&input[cursor..start + prefix.len()]);
        output.push_str(replacement);
        cursor = start + prefix.len();
        while cursor < input.len() && !is_cookie_delimiter(input.as_bytes()[cursor]) {
            cursor += 1;
        }
    }
    output.push_str(&input[cursor..]);
    output
}

fn is_url_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'&' | b'"' | b'\'' | b'<' | b'>' | b')' | b' ' | b'\n' | b'\t'
    )
}

fn is_cookie_delimiter(byte: u8) -> bool {
    matches!(byte, b';' | b'"' | b'\'' | b' ' | b'\n' | b'\t')
}
