// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, html};

#[derive(Clone, Copy)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Clone, Copy)]
pub enum HeadingSize {
    Xs,
    Sm,
    Base,
    Lg,
    Xl,
    Xl2,
    Xl3,
    Xl4,
}

fn default_heading_classes(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "text-3xl font-bold",
        HeadingLevel::H2 => "text-2xl font-semibold",
        HeadingLevel::H3 => "text-xl font-semibold",
        HeadingLevel::H4 => "text-lg font-semibold",
        HeadingLevel::H5 => "text-base font-semibold",
        HeadingLevel::H6 => "text-sm font-semibold",
    }
}

fn custom_size_class(size: HeadingSize) -> &'static str {
    match size {
        HeadingSize::Xs => "text-xs",
        HeadingSize::Sm => "text-sm",
        HeadingSize::Base => "text-base",
        HeadingSize::Lg => "text-lg",
        HeadingSize::Xl => "text-xl",
        HeadingSize::Xl2 => "text-2xl",
        HeadingSize::Xl3 => "text-3xl",
        HeadingSize::Xl4 => "text-4xl",
    }
}

pub fn heading(level: HeadingLevel, size: Option<HeadingSize>, content: Markup) -> Markup {
    let size_classes = match size {
        Some(s) => custom_size_class(s),
        None => default_heading_classes(level),
    };
    let classes = format!("text-gray-900 tracking-tight {size_classes}");
    match level {
        HeadingLevel::H1 => html! { h1 class=(classes) { (content) } },
        HeadingLevel::H2 => html! { h2 class=(classes) { (content) } },
        HeadingLevel::H3 => html! { h3 class=(classes) { (content) } },
        HeadingLevel::H4 => html! { h4 class=(classes) { (content) } },
        HeadingLevel::H5 => html! { h5 class=(classes) { (content) } },
        HeadingLevel::H6 => html! { h6 class=(classes) { (content) } },
    }
}

#[derive(Clone, Copy)]
pub enum TextSize {
    Xs,
    Sm,
    Base,
    Lg,
}

#[derive(Clone, Copy)]
pub enum TextColor {
    Default,
    Muted,
    Primary,
    Danger,
    Success,
}

pub fn text(size: TextSize, color: TextColor, content: Markup) -> Markup {
    let size_class = match size {
        TextSize::Xs => "text-xs",
        TextSize::Sm => "text-sm",
        TextSize::Base => "text-base",
        TextSize::Lg => "text-lg",
    };
    let color_class = match color {
        TextColor::Default => "text-gray-900",
        TextColor::Muted => "text-neutral-500",
        TextColor::Primary => "text-brand-primary",
        TextColor::Danger => "text-red-600",
        TextColor::Success => "text-green-600",
    };
    html! {
        p class={(size_class) " font-normal " (color_class)} { (content) }
    }
}

pub fn caption(content: Markup) -> Markup {
    html! {
        p class="text-xs text-gray-500" { (content) }
    }
}

#[derive(Clone, Copy)]
pub enum CaptionVariant {
    Default,
    Error,
    Success,
}

pub fn caption_variant(variant: CaptionVariant, content: Markup) -> Markup {
    let color = match variant {
        CaptionVariant::Default => "text-gray-500",
        CaptionVariant::Error => "text-red-600",
        CaptionVariant::Success => "text-green-600",
    };
    html! {
        p class={"text-xs " (color)} { (content) }
    }
}

pub fn label_text(for_id: Option<&str>, required: bool, content: Markup) -> Markup {
    let classes = "block text-xs font-semibold uppercase tracking-wide text-neutral-500";
    html! {
        label for=[for_id] class=(classes) {
            (content)
            @if required {
                span class="ml-1 text-red-600" { "*" }
            }
        }
    }
}

pub fn section_heading(title: &str, actions: Option<Markup>) -> Markup {
    match actions {
        Some(action_content) => html! {
            div class="mb-4 flex items-center justify-between" {
                h2 class="font-semibold text-gray-900 text-xl" { (title) }
                div class="flex items-center gap-2" { (action_content) }
            }
        },
        None => html! {
            h2 class="mb-4 font-semibold text-gray-900 text-xl" { (title) }
        },
    }
}

pub fn inline_text(content: &str) -> Markup {
    html! { span { (content) } }
}
