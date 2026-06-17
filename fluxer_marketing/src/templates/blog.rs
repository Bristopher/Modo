// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    OgType, PageMeta, content_layout, format_long_date, heading_anchor_script,
    icons::{Icon, icon},
    tr,
};
use crate::{
    content::{
        BLOG_POSTS, BLOG_TAGS, BlogPost, blog_post_body, blog_tag_label, blog_tag_slug,
        get_blog_tag, render_blog_markdown_with_copy_label,
    },
    i18n::{MarketingI18n, descriptors::*},
    request_context::RequestContext,
};
use maud::{Markup, html};

const HAMPUS_AVATAR_PATH: &str = "/blog/assets/hampus-kraft-avatar.jpg";

pub fn blog_page(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    query: Option<&str>,
    tag: Option<&str>,
) -> Markup {
    let query = query.unwrap_or_default().trim().to_owned();
    let selected_tag = tag.and_then(resolve_blog_tag);
    let selected_tag_label = selected_tag.map(|tag| blog_tag_label(i18n, ctx.locale, tag));
    let posts = BLOG_POSTS
        .iter()
        .copied()
        .filter(|post| blog_post_matches(i18n, ctx, *post, &query, selected_tag))
        .collect::<Vec<_>>();
    let is_filtered = !query.is_empty() || selected_tag.is_some();
    content_layout(
        i18n,
        ctx,
        PageMeta {
            title: tr(i18n, ctx, BLOG_TITLE_DESCRIPTOR),
            description: tr(i18n, ctx, BLOG_DESCRIPTION_DESCRIPTOR),
            ..Default::default()
        },
        html! {
            section class="mx-auto max-w-5xl" {
                header class="mb-10 grid gap-6 lg:grid-cols-[minmax(0,1fr)_260px] lg:items-end" {
                    div class="max-w-3xl" {
                        h1 class="display text-4xl text-gray-950 md:text-5xl" {
                            (tr(i18n, ctx, BLOG_TITLE_DESCRIPTOR))
                        }
                        p class="mt-5 text-lg leading-relaxed text-muted-foreground" {
                            (tr(i18n, ctx, BLOG_DESCRIPTION_DESCRIPTOR))
                        }
                    }
                    aside class="rounded-lg border border-gray-200 bg-gray-50 p-5" {
                        h2 class="font-semibold text-gray-950 text-sm" {
                            (tr(i18n, ctx, BLOG_FEEDS_DESCRIPTOR))
                        }
                        div class="mt-3 flex flex-col gap-2" {
                            (blog_feed_link(ctx, "/blog/rss.xml", tr(i18n, ctx, BLOG_RSS_FEED_DESCRIPTOR)))
                            (blog_feed_link(ctx, "/blog/atom.xml", tr(i18n, ctx, BLOG_ATOM_FEED_DESCRIPTOR)))
                        }
                    }
                }

                div class="mb-8 grid gap-4 lg:grid-cols-[minmax(0,1fr)_260px]" {
                    form role="search" action=(ctx.href("/blog")) method="get" class="min-w-0" {
                        @if let Some(tag) = selected_tag {
                            input type="hidden" name="tag" value=(blog_tag_slug(tag));
                        }
                        label for="blog-search-input" class="sr-only" {
                            (tr(i18n, ctx, BLOG_SEARCH_PLACEHOLDER_DESCRIPTOR))
                        }
                        div class="flex min-h-14 items-stretch overflow-hidden rounded-lg border border-gray-200 bg-white p-1.5 shadow-xl shadow-gray-950/5 ring-1 ring-gray-950/5 transition-colors focus-within:border-gray-300" {
                            div class="pointer-events-none flex shrink-0 items-center justify-center pr-2 pl-3 text-primary" {
                                (icon(Icon::MagnifyingGlassBold, "h-5 w-5"))
                            }
                            input
                                id="blog-search-input"
                                name="q"
                                type="search"
                                value=(query)
                                placeholder=(tr(i18n, ctx, BLOG_SEARCH_PLACEHOLDER_DESCRIPTOR))
                                class="min-w-0 flex-1 bg-transparent py-2 pr-3 text-base font-medium text-gray-950 outline-none placeholder:text-gray-400";
                            button type="submit" class="inline-flex min-w-24 shrink-0 items-center justify-center rounded-lg bg-primary px-5 py-2 font-semibold text-sm text-white transition hover:bg-primary-600 focus:outline-none" {
                                (tr(i18n, ctx, BLOG_SEARCH_BUTTON_DESCRIPTOR))
                            }
                        }
                    }
                    nav aria-label=(tr(i18n, ctx, BLOG_TAGS_DESCRIPTOR)) class="flex flex-wrap items-center gap-2 lg:justify-end" {
                        (blog_tag_chip(ctx, tr(i18n, ctx, BLOG_ALL_POSTS_DESCRIPTOR), "/blog".to_owned(), selected_tag.is_none()))
                        @for tag in BLOG_TAGS {
                            (blog_tag_chip(ctx, blog_tag_label(i18n, ctx.locale, tag), format!("/blog?tag={}", blog_tag_slug(tag)), selected_tag == Some(*tag)))
                        }
                    }
                }

                @if is_filtered {
                    div class="mb-6 flex flex-wrap items-center gap-3" {
                        h2 class="display text-2xl text-gray-950 md:text-3xl" {
                            (tr(i18n, ctx, BLOG_SEARCH_RESULTS_DESCRIPTOR))
                        }
                        @if let Some(tag_label) = &selected_tag_label {
                            span class="rounded-full bg-indigo-50 px-3 py-1 font-medium text-indigo-700 text-sm" {
                                (i18n.text_with(ctx.locale, BLOG_FILTERED_BY_TAG_DESCRIPTOR, &[("tag", tag_label.as_str())]))
                            }
                        }
                    }
                } @else {
                    h2 class="display mb-6 text-2xl text-gray-950 md:text-3xl" {
                        (tr(i18n, ctx, BLOG_ALL_POSTS_DESCRIPTOR))
                    }
                }

                @if posts.is_empty() {
                    div class="rounded-lg border border-gray-200 bg-gray-50 p-6" {
                        h3 class="font-semibold text-gray-950 text-lg" {
                            (tr(i18n, ctx, BLOG_NO_RESULTS_TITLE_DESCRIPTOR))
                        }
                        p class="mt-2 text-gray-600" {
                            (tr(i18n, ctx, BLOG_NO_RESULTS_DESCRIPTION_DESCRIPTOR))
                        }
                    }
                } @else {
                    div class="grid gap-4 lg:grid-cols-2" {
                        @for post in &posts {
                            (blog_post_card(i18n, ctx, *post))
                        }
                    }
                }
            }
        },
    )
}

pub fn blog_post_page(i18n: &MarketingI18n, ctx: &RequestContext, post: BlogPost) -> Markup {
    let copy_link_label = tr(i18n, ctx, NAVIGATION_COPY_LINK_TO_SECTION_DESCRIPTOR);
    let linked_article_label = tr(i18n, ctx, BLOG_LINKED_ARTICLE_DESCRIPTOR);
    let local_markdown_base = ctx.href("/");
    let rendered = render_blog_markdown_with_copy_label(
        post.body,
        &local_markdown_base,
        &copy_link_label,
        &linked_article_label,
    );
    let related = collect_related_blog_posts(post);
    let title = tr(i18n, ctx, post.title);
    let description = tr(i18n, ctx, post.description);
    content_layout(
        i18n,
        ctx,
        PageMeta {
            title: title.clone(),
            description: description.clone(),
            og_type: OgType::Article,
            og_image_url: Some(ctx.absolute_href(post.feature_image_path)),
            published_time: Some(post.published_at.to_owned()),
            modified_time: Some(post.updated_at.to_owned()),
            author: Some(post.author.to_owned()),
            article_tags: post
                .tags
                .iter()
                .map(|tag| blog_tag_label(i18n, ctx.locale, tag))
                .collect(),
            json_ld: vec![blog_post_json_ld(i18n, ctx, post)],
            ..Default::default()
        },
        html! {
            section class="mx-auto max-w-3xl" {
                div class="mb-5" {
                    a href=(ctx.href("/blog")) class="text-muted-foreground text-sm transition-colors hover:text-foreground" {
                        "← " (tr(i18n, ctx, BLOG_BACK_TO_BLOG_DESCRIPTOR))
                    }
                }
                header class="mb-8 space-y-5" {
                    div class="flex flex-wrap items-center gap-2" {
                        @for tag in post.tags {
                            a href=(ctx.href(&format!("/blog?tag={}", blog_tag_slug(tag)))) class="rounded-full bg-indigo-50 px-3 py-1 font-medium text-indigo-700 text-xs transition hover:bg-indigo-100" {
                                (blog_tag_label(i18n, ctx.locale, tag))
                            }
                        }
                    }
                    h1 class="font-bold text-3xl text-foreground leading-tight md:text-4xl" {
                        (title)
                    }
                    p class="text-base leading-relaxed text-muted-foreground md:text-lg" {
                        (description)
                    }
                    div class="flex items-center gap-3 text-sm" {
                        img
                            class="h-14 w-14 rounded-full border border-gray-200 object-cover"
                            src=(ctx.href(HAMPUS_AVATAR_PATH))
                            alt=(post.author)
                            width="56"
                            height="56"
                            loading="eager"
                            decoding="async";
                        div class="min-w-0" {
                            div class="font-semibold text-gray-950" { (post.author) }
                            div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-muted-foreground" {
                                span { (format_long_date(date_part(post.published_at), ctx.locale)) }
                            }
                        }
                    }
                }
                (blog_feature_image(
                    i18n,
                    ctx,
                    post,
                    "mb-9 aspect-video w-full overflow-hidden rounded-lg border border-gray-200",
                    "h-full w-full object-cover",
                    "eager",
                    Some("high"),
                    "(max-width: 768px) 100vw, 768px",
                ))
                article id="policy-content" class="min-w-0" { (rendered) }
                @if !related.is_empty() {
                    div class="mt-10 border-gray-200/60 border-t pt-7" {
                        h2 class="mb-3 font-semibold text-foreground text-base" {
                            (tr(i18n, ctx, BLOG_RELATED_POSTS_DESCRIPTOR))
                        }
                        div class="grid gap-3" {
                            @for entry in &related {
                                (related_blog_post(i18n, ctx, *entry))
                            }
                        }
                    }
                }
                (heading_anchor_script())
            }
        },
    )
}

fn blog_feed_link(ctx: &RequestContext, href: &str, label: String) -> Markup {
    html! {
        a href=(ctx.href(href)) class="inline-flex items-center gap-2 text-sm font-medium text-primary hover:text-primary-600" {
            (icon(Icon::Rss, "h-4 w-4"))
            span { (label) }
        }
    }
}

fn blog_tag_chip(ctx: &RequestContext, label: String, href: String, active: bool) -> Markup {
    let class_name = if active {
        "rounded-full bg-primary px-3 py-1.5 font-semibold text-sm text-white"
    } else {
        "rounded-full border border-gray-200 bg-white px-3 py-1.5 font-semibold text-gray-700 text-sm transition hover:border-primary/40 hover:text-primary"
    };
    html! {
        a href=(ctx.href(&href)) class=(class_name) { (label) }
    }
}

fn blog_post_card(i18n: &MarketingI18n, ctx: &RequestContext, post: BlogPost) -> Markup {
    html! {
        article class="group overflow-hidden rounded-lg border border-gray-200 bg-white transition-colors hover:border-primary/40" {
            a href=(blog_post_url(ctx, post)) class="block focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2" {
                (blog_feature_image(
                    i18n,
                    ctx,
                    post,
                    "aspect-video w-full overflow-hidden",
                    "h-full w-full object-cover",
                    "lazy",
                    None,
                    "(max-width: 1024px) 100vw, 50vw",
                ))
                div class="p-5" {
                    div class="mb-3 flex flex-wrap items-center gap-2 text-xs text-gray-500" {
                        @for tag in post.tags {
                            span class="rounded-full bg-indigo-50 px-2.5 py-1 font-medium text-indigo-700" { (blog_tag_label(i18n, ctx.locale, tag)) }
                        }
                        span { (format_long_date(date_part(post.published_at), ctx.locale)) }
                    }
                    h3 class="font-semibold text-xl text-gray-950 transition group-hover:text-primary" {
                        (tr(i18n, ctx, post.title))
                    }
                    p class="mt-2 text-sm leading-relaxed text-gray-600" {
                        (tr(i18n, ctx, post.description))
                    }
                    div class="mt-4 inline-flex items-center gap-1.5 font-semibold text-primary text-sm" {
                        (tr(i18n, ctx, BLOG_READ_ARTICLE_DESCRIPTOR))
                        (icon(Icon::ArrowRight, "h-3.5 w-3.5"))
                    }
                }
            }
        }
    }
}

fn related_blog_post(i18n: &MarketingI18n, ctx: &RequestContext, post: BlogPost) -> Markup {
    html! {
        a href=(blog_post_url(ctx, post)) class="group block py-2 text-muted-foreground text-sm hover:text-foreground" {
            div class="font-medium text-foreground group-hover:text-primary" { (tr(i18n, ctx, post.title)) }
            div class="mt-0.5 text-muted-foreground text-sm" { (tr(i18n, ctx, post.description)) }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn blog_feature_image(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    post: BlogPost,
    frame_class: &str,
    image_class: &str,
    loading: &str,
    fetch_priority: Option<&str>,
    sizes: &str,
) -> Markup {
    let placeholder_style = format!(
        "--blog-image-placeholder: url('{}')",
        post.feature_image_placeholder
    );
    html! {
        div class=(format!("blog-image-frame {frame_class}")) style=(placeholder_style) {
            picture {
                source type="image/avif" srcset=(responsive_srcset(ctx, post, "avif")) sizes=(sizes);
                source type="image/webp" srcset=(responsive_srcset(ctx, post, "webp")) sizes=(sizes);
                source type="image/jpeg" srcset=(responsive_srcset(ctx, post, "jpg")) sizes=(sizes);
                img
                    class=(image_class)
                    src=(ctx.href(post.feature_image_path))
                    alt=(tr(i18n, ctx, post.feature_image_alt))
                    loading=(loading)
                    decoding="async"
                    fetchpriority=[fetch_priority];
            }
        }
    }
}

fn responsive_srcset(ctx: &RequestContext, post: BlogPost, extension: &str) -> String {
    [640, 960, 1280, 2000]
        .iter()
        .map(|width| {
            format!(
                "{} {width}w",
                ctx.href(&format!(
                    "{}-{width}.{extension}",
                    post.feature_image_base_path
                ))
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn blog_post_matches(
    i18n: &MarketingI18n,
    ctx: &RequestContext,
    post: BlogPost,
    query: &str,
    tag: Option<&str>,
) -> bool {
    if let Some(tag) = tag
        && !post.tags.contains(&tag)
    {
        return false;
    }
    if query.is_empty() {
        return true;
    }
    let query = query.to_lowercase();
    tr(i18n, ctx, post.title).to_lowercase().contains(&query)
        || tr(i18n, ctx, post.description)
            .to_lowercase()
            .contains(&query)
        || post.author.to_lowercase().contains(&query)
        || post.tags.iter().any(|tag| {
            blog_tag_label(i18n, ctx.locale, tag)
                .to_lowercase()
                .contains(&query)
        })
        || blog_post_body(post.body).to_lowercase().contains(&query)
}

fn collect_related_blog_posts(post: BlogPost) -> Vec<BlogPost> {
    let others = BLOG_POSTS
        .iter()
        .copied()
        .filter(|entry| entry.slug != post.slug)
        .collect::<Vec<_>>();
    let mut same_tag = others
        .iter()
        .copied()
        .filter(|entry| entry.tags.iter().any(|tag| post.tags.contains(tag)))
        .collect::<Vec<_>>();
    let mut fallback = others
        .into_iter()
        .filter(|entry| !entry.tags.iter().any(|tag| post.tags.contains(tag)))
        .collect::<Vec<_>>();
    same_tag.append(&mut fallback);
    same_tag.truncate(3);
    same_tag
}

fn resolve_blog_tag(value: &str) -> Option<&'static str> {
    get_blog_tag(&blog_tag_slug(value))
}

fn blog_post_url(ctx: &RequestContext, post: BlogPost) -> String {
    ctx.href(&blog_post_path(post))
}

fn blog_post_absolute_url(ctx: &RequestContext, post: BlogPost) -> String {
    ctx.absolute_href(&blog_post_path(post))
}

fn blog_post_path(post: BlogPost) -> String {
    format!("/blog/{}", post.slug)
}

fn date_part(iso: &str) -> &str {
    iso.split_once('T').map(|(date, _)| date).unwrap_or(iso)
}

fn blog_post_json_ld(i18n: &MarketingI18n, ctx: &RequestContext, post: BlogPost) -> String {
    let title = tr(i18n, ctx, post.title);
    let description = tr(i18n, ctx, post.description);
    let keywords = post
        .tags
        .iter()
        .map(|tag| blog_tag_label(i18n, ctx.locale, tag))
        .collect::<Vec<_>>()
        .join(", ");
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": title,
        "description": description,
        "url": blog_post_absolute_url(ctx, post),
        "image": ctx.absolute_href(post.feature_image_path),
        "datePublished": post.published_at,
        "dateModified": post.updated_at,
        "author": {
            "@type": "Person",
            "name": post.author,
        },
        "publisher": {
            "@type": "Organization",
            "name": "Fluxer",
            "url": ctx.base_url.as_str(),
        },
        "mainEntityOfPage": {
            "@type": "WebPage",
            "@id": blog_post_absolute_url(ctx, post),
        },
        "keywords": keywords,
    })
    .to_string()
}
