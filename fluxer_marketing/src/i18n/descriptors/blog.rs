// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const BLOG_ALL_POSTS_DESCRIPTOR = {
        key: "blog.all_posts",
        message: "All posts",
        comment: "Short filter label on the blog index. It clears tag filters and shows every blog post.",
    };
);

crate::marketing_message!(
    pub const BLOG_ATOM_FEED_DESCRIPTOR = {
        key: "blog.atom_feed",
        message: "Atom feed",
        comment: "Compact link label for the Atom feed on the blog index. Keep the feed protocol name in conventional form.",
    };
);

crate::marketing_message!(
    pub const BLOG_BACK_TO_BLOG_DESCRIPTOR = {
        key: "blog.back_to_blog",
        message: "Back to blog",
        comment: "Back-link label on a blog article page. It returns readers to the blog index.",
    };
);

crate::marketing_message!(
    pub const BLOG_DESCRIPTION_DESCRIPTOR = {
        key: "blog.description",
        message: "Updates, roadmap notes, and engineering write-ups from the {product_name} team.",
        comment: "Blog index meta description and intro copy. Keep it concise and editorial, not sales-oriented. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_FEEDS_DESCRIPTOR = {
        key: "blog.feeds",
        message: "Feeds",
        comment: "Short heading for RSS and Atom links on the blog index. Keep it compact.",
    };
);

crate::marketing_message!(
    pub const BLOG_LINKED_ARTICLE_DESCRIPTOR = {
        key: "blog.linked_article",
        message: "Linked article",
        comment: "Fallback source label inside a blog bookmark card when the linked page has no readable source or publisher. Keep it short and neutral.",
    };
);

crate::marketing_message!(
    pub const BLOG_FILTERED_BY_TAG_DESCRIPTOR = {
        key: "blog.filtered_by_tag",
        message: "Filtered by {tag}",
        comment: "Blog index status text shown when a tag filter is active. Preserve {tag}; it is the visible tag name. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_POST_HOW_I_BUILT_FLUXER_TITLE_DESCRIPTOR = {
        key: "blog.post.how_i_built_fluxer.title",
        message: "How I built {product_name}, a {discord}-like chat app",
        comment: "Blog article title shown in cards, article pages, metadata, and feeds. Keep Fluxer and Discord recognizable as product names. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_POST_MOBILE_CLIENTS_AND_FLUXER_V2_DESCRIPTION_DESCRIPTOR = {
        key: "blog.post.mobile_clients_and_fluxer_v2.description",
        message: "{product_name} v2 is out, mobile clients are open source, self-hosting is improving, and public development is moving back to GitHub.",
        comment: "Blog article summary shown in cards, article pages, metadata, and feeds. Keep Fluxer as a product name, v2 as the release name, and GitHub as the product name. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_POST_MOBILE_CLIENTS_AND_FLUXER_V2_TITLE_DESCRIPTOR = {
        key: "blog.post.mobile_clients_and_fluxer_v2.title",
        message: "Mobile clients and {product_name} v2",
        comment: "Blog article title shown in cards, article pages, metadata, and feeds. Keep Fluxer as a product name and v2 as the release name. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_POST_ROADMAP_2026_DESCRIPTION_DESCRIPTOR = {
        key: "blog.post.roadmap_2026.description",
        message: "The current 2026 roadmap for {product_name}: canary, mobile, self-hosting, federation, voice and video, and the backend reliability work behind it.",
        comment: "Blog article summary shown in cards, article pages, metadata, and feeds. Keep Fluxer as a product name and canary as the release-channel name. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_POST_ROADMAP_2026_TITLE_DESCRIPTOR = {
        key: "blog.post.roadmap_2026.title",
        message: "Roadmap 2026",
        comment: "Blog article title shown in cards, article pages, metadata, and feeds. Refers to Fluxer's 2026 product roadmap.",
    };
);

crate::marketing_message!(
    pub const BLOG_NO_RESULTS_DESCRIPTION_DESCRIPTOR = {
        key: "blog.no_results.description",
        message: "Try another search term or clear the current filters.",
        comment: "Empty-state body on the blog index when search and tag filters find no posts.",
    };
);

crate::marketing_message!(
    pub const BLOG_NO_RESULTS_TITLE_DESCRIPTOR = {
        key: "blog.no_results.title",
        message: "No posts found",
        comment: "Empty-state heading on the blog index when search and tag filters find no posts.",
    };
);

crate::marketing_message!(
    pub const BLOG_PUBLISHED_DESCRIPTOR = {
        key: "blog.published",
        message: "Published",
        comment: "Short metadata label on a blog article page. It precedes the article publication date.",
    };
);

crate::marketing_message!(
    pub const BLOG_READ_ARTICLE_DESCRIPTOR = {
        key: "blog.read_article",
        message: "Read article",
        comment: "Call-to-action label on blog post cards. Keep it concise and neutral.",
    };
);

crate::marketing_message!(
    pub const BLOG_RELATED_POSTS_DESCRIPTOR = {
        key: "blog.related_posts",
        message: "Related posts",
        comment: "Section heading under a blog article showing other posts readers may open next.",
    };
);

crate::marketing_message!(
    pub const BLOG_RSS_FEED_DESCRIPTOR = {
        key: "blog.rss_feed",
        message: "RSS feed",
        comment: "Compact link label for the RSS feed on the blog index. Keep the feed protocol name in conventional form.",
    };
);

crate::marketing_message!(
    pub const BLOG_SEARCH_BUTTON_DESCRIPTOR = {
        key: "blog.search.button",
        message: "Search",
        comment: "Button label for the blog search form. Keep it short and action-oriented.",
    };
);

crate::marketing_message!(
    pub const BLOG_SEARCH_PLACEHOLDER_DESCRIPTOR = {
        key: "blog.search.placeholder",
        message: "Search blog posts…",
        comment: "Placeholder text in the blog search input. Keep it concise and clearly scoped to blog posts.",
    };
);

crate::marketing_message!(
    pub const BLOG_SEARCH_RESULTS_DESCRIPTOR = {
        key: "blog.search.results",
        message: "Search results",
        comment: "Section heading on the blog index when a search query or tag filter is active.",
    };
);

crate::marketing_message!(
    pub const BLOG_TAG_NEWS_DESCRIPTOR = {
        key: "blog.tag.news",
        message: "News",
        comment: "Blog tag label for product updates and announcements. Keep it short because it appears in filter chips and metadata.",
    };
);

crate::marketing_message!(
    pub const BLOG_TAGS_DESCRIPTOR = {
        key: "blog.tags",
        message: "Tags",
        comment: "Short label for the list of blog tags. Keep it compact.",
    };
);

crate::marketing_message!(
    pub const BLOG_TITLE_DESCRIPTOR = {
        key: "blog.title",
        message: "{product_name} Blog",
        comment: "Blog index title and meta title. Keep the product name and blog label recognizable. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const BLOG_UPDATED_DESCRIPTOR = {
        key: "blog.updated",
        message: "Updated",
        comment: "Short metadata label on a blog article page. It precedes the article update date.",
    };
);
