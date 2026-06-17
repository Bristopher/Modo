// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, PreEscaped, html};

pub fn paperclip_icon(color: &str) -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"
            class={"inline-block h-3 w-3 " (color)}
        {
            (PreEscaped(concat!(
                r#"<rect width="256" height="256" fill="none"/>"#,
                r#"<path d="M108.71,197.23l-5.11,5.11a46.63,46.63,0,0,1-66-.05h0"#,
                r#"a46.63,46.63,0,0,1,.06-65.89L72.4,101.66a46.62,46.62,0,0,1,65.94,0"#,
                r#"h0A46.34,46.34,0,0,1,150.78,124" fill="none" stroke="currentColor""#,
                r#" stroke-linecap="round" stroke-linejoin="round" stroke-width="24"/>"#,
                r#"<path d="M147.29,58.77l5.11-5.11a46.62,46.62,0,0,1,65.94,0h0"#,
                r#"a46.62,46.62,0,0,1,0,65.94L193.94,144,183.6,154.34"#,
                r#"a46.63,46.63,0,0,1-66-.05h0A46.46,46.46,0,0,1,105.22,132""#,
                r#" fill="none" stroke="currentColor" stroke-linecap="round""#,
                r#" stroke-linejoin="round" stroke-width="24"/>"#,
            )))
        }
    }
}

pub fn checkmark_icon(color: &str) -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"
            class={"inline-block h-4 w-4 " (color)}
        {
            (PreEscaped(concat!(
                r#"<rect width="256" height="256" fill="none"/>"#,
                r#"<polyline points="40 144 96 200 224 72" fill="none""#,
                r#" stroke="currentColor" stroke-linecap="round""#,
                r#" stroke-linejoin="round" stroke-width="24"/>"#,
            )))
        }
    }
}

pub fn x_icon(color: &str) -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"
            class={"inline-block h-4 w-4 " (color)}
        {
            (PreEscaped(concat!(
                r#"<rect width="256" height="256" fill="none"/>"#,
                r#"<line x1="200" y1="56" x2="56" y2="200" fill="none""#,
                r#" stroke="currentColor" stroke-linecap="round""#,
                r#" stroke-linejoin="round" stroke-width="24"/>"#,
                r#"<line x1="200" y1="200" x2="56" y2="56" fill="none""#,
                r#" stroke="currentColor" stroke-linecap="round""#,
                r#" stroke-linejoin="round" stroke-width="24"/>"#,
            )))
        }
    }
}

pub fn close_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            class="h-5 w-5" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true"
        {
            (PreEscaped(concat!(
                r#"<line x1="18" y1="6" x2="6" y2="18"/>"#,
                r#"<line x1="6" y1="6" x2="18" y2="18"/>"#,
            )))
        }
    }
}

pub fn spinner_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round"
            class="h-6 w-6 animate-spin" aria-hidden="true"
        {
            (PreEscaped(r#"<path d="M21 12a9 9 0 1 1-6.219-8.56"/>"#))
        }
    }
}

pub fn search_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            class="h-4 w-4" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true"
        {
            (PreEscaped(concat!(
                r#"<circle cx="11" cy="11" r="8"/>"#,
                r#"<line x1="21" y1="21" x2="16.65" y2="16.65"/>"#,
            )))
        }
    }
}

pub fn chevron_right_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            class="h-4 w-4" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true"
        {
            (PreEscaped(r#"<polyline points="9 18 15 12 9 6"/>"#))
        }
    }
}

pub fn external_link_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            class="h-3.5 w-3.5" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true"
        {
            (PreEscaped(concat!(
                r#"<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8"#,
                r#"a2 2 0 0 1 2-2h6"/>"#,
                r#"<polyline points="15 3 21 3 21 9"/>"#,
                r#"<line x1="10" y1="14" x2="21" y2="3"/>"#,
            )))
        }
    }
}

pub fn copy_icon() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
            class="h-3.5 w-3.5" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true"
        {
            (PreEscaped(concat!(
                r#"<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>"#,
                r#"<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9"#,
                r#"a2 2 0 0 1 2 2v1"/>"#,
            )))
        }
    }
}
