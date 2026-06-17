// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::Cursor;

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("svg sanitization failed")]
    Sanitize,
}

const MAX_SANITIZED_BYTES: usize = 16 * 1024 * 1024;

pub fn sanitize(input: &[u8]) -> Result<Vec<u8>, SvgError> {
    let filter = svg_hush::Filter::new();
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    filter
        .filter(Cursor::new(input), &mut out)
        .map_err(|_| SvgError::Sanitize)?;
    if out.len() > MAX_SANITIZED_BYTES {
        return Err(SvgError::Sanitize);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_inline_script_elements() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
            <script>alert(1)</script>
            <rect width="10" height="10" fill="red"/>
        </svg>"#;
        let cleaned = sanitize(svg).expect("sanitize ok");
        let text = std::str::from_utf8(&cleaned).unwrap();
        assert!(!text.contains("<script"), "script tag survived: {text}");
        assert!(text.contains("<rect"), "rect element missing: {text}");
    }

    #[test]
    fn drops_event_handler_attributes() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)">
            <rect width="1" height="1" onclick="alert(2)"/>
        </svg>"#;
        let cleaned = sanitize(svg).expect("sanitize ok");
        let text = std::str::from_utf8(&cleaned).unwrap();
        assert!(!text.to_ascii_lowercase().contains("onload"));
        assert!(!text.to_ascii_lowercase().contains("onclick"));
    }

    #[test]
    fn neutralizes_javascript_hrefs() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg">
            <a xlink:href="javascript:alert(1)" href="javascript:alert(2)">
                <rect width="1" height="1"/>
            </a>
        </svg>"#;
        if let Ok(cleaned) = sanitize(svg) {
            let text = std::str::from_utf8(&cleaned).unwrap();
            assert!(
                !text.to_ascii_lowercase().contains("javascript:"),
                "javascript: scheme survived sanitization: {text}"
            );
        }
    }

    #[test]
    fn preserves_basic_shapes_and_paths() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
            <path d="M0 0 L10 10 Z" fill="#abc"/>
            <circle cx="5" cy="5" r="3"/>
        </svg>"##;
        let cleaned = sanitize(svg).expect("sanitize ok");
        let text = std::str::from_utf8(&cleaned).unwrap();
        assert!(text.contains("<svg"));
        assert!(text.contains("<path"));
        assert!(text.contains("<circle"));
    }

    #[test]
    fn rejects_invalid_xml() {
        let result = sanitize(b"<svg>this is < not valid </svg");
        assert!(result.is_err());
    }
}
