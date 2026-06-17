// SPDX-License-Identifier: AGPL-3.0-or-later

use maud::{Markup, html};

#[derive(Clone, Copy)]
pub enum StackGap {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy)]
pub enum StackAlign {
    Start,
    Center,
    End,
    Stretch,
}

fn gap_class(gap: StackGap) -> &'static str {
    match gap {
        StackGap::Sm => "gap-2",
        StackGap::Md => "gap-4",
        StackGap::Lg => "gap-6",
    }
}

fn align_class(align: StackAlign) -> &'static str {
    match align {
        StackAlign::Start => "items-start",
        StackAlign::Center => "items-center",
        StackAlign::End => "items-end",
        StackAlign::Stretch => "items-stretch",
    }
}

pub fn vstack(gap: StackGap, align: StackAlign, content: Markup) -> Markup {
    let classes = format!("flex flex-col {} {}", gap_class(gap), align_class(align),);
    html! {
        div class=(classes) { (content) }
    }
}

pub fn hstack(gap: StackGap, align: StackAlign, content: Markup) -> Markup {
    let classes = format!("flex flex-row {} {}", gap_class(gap), align_class(align),);
    html! {
        div class=(classes) { (content) }
    }
}

pub fn stack(content: Markup) -> Markup {
    vstack(StackGap::Md, StackAlign::Stretch, content)
}
