// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_BLUESKY_FOLLOW_US_DESCRIPTOR = {
        key: "social_and_feeds.bluesky.follow_us",
        message: "Follow us on {bluesky}",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_BLUESKY_LABEL_DESCRIPTOR = {
        key: "social_and_feeds.bluesky.label",
        message: "{bluesky}",
        comment: "Short UI label or heading in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_BLUESKY_RSS_FEED_DESCRIPTOR = {
        key: "social_and_feeds.bluesky.rss_feed",
        message: "{bluesky} RSS feed",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_FOLLOW_FLUXER_DESCRIPTOR = {
        key: "social_and_feeds.follow_fluxer",
        message: "Follow @{social_handle}",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_GITHUB_DESCRIPTOR = {
        key: "social_and_feeds.github",
        message: "{github}",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_RSS_BLOG_RSS_FEED_DESCRIPTOR = {
        key: "social_and_feeds.rss.blog_rss_feed",
        message: "Blog RSS feed",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_RSS_FLUXER_BLOG_RSS_DESCRIPTOR = {
        key: "social_and_feeds.rss.fluxer_blog_rss",
        message: "{product_name} blog RSS",
        comment: "Compact UI label in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_RSS_LABEL_DESCRIPTOR = {
        key: "social_and_feeds.rss.label",
        message: "RSS feed",
        comment: "Short UI label or heading in social, Bluesky, GitHub, or RSS follow sections. Keep labels recognizable, avoid dangling sentence fragments, and preserve handles/placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const SOCIAL_AND_FEEDS_STAY_UPDATED_CTA_DESCRIPTOR = {
        key: "social_and_feeds.stay_updated_cta",
        message: "Stay updated on news, service status, and what's happening. You can also subscribe to our",
        comment: "Body copy in the social/follow card. It is followed by separate RSS feed links in the UI, so translate as an unfinished lead-in only if that grammar works in the target locale.",
    };
);
