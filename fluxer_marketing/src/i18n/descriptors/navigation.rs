// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const NAVIGATION_CLOSE_DESCRIPTOR = {
        key: "navigation.close",
        message: "Close",
        comment: "Short UI label or heading in site navigation, mobile drawer controls, or accessibility labels. Keep it brief and unambiguous.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_CLOSE_NAVIGATION_MENU_DESCRIPTOR = {
        key: "navigation.close_navigation_menu",
        message: "Close navigation menu",
        comment: "Short UI label or heading in site navigation, mobile drawer controls, or accessibility labels. Keep it brief and unambiguous.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_OPEN_NAVIGATION_MENU_DESCRIPTOR = {
        key: "navigation.open_navigation_menu",
        message: "Open navigation menu",
        comment: "Accessibility label for the mobile navigation menu toggle. Keep it brief and unambiguous.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_COPY_LINK_TO_SECTION_DESCRIPTOR = {
        key: "navigation.copy_link_to_section",
        message: "Copy link to section",
        comment: "ARIA label for the small link icon shown beside policy and job content headings. It copies a direct URL to that section; keep it concise.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_GO_HOME_DESCRIPTOR = {
        key: "navigation.go_home",
        message: "Go home",
        comment: "Compact UI label in site navigation, mobile drawer controls, or accessibility labels. Keep it brief and unambiguous.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_ON_THIS_PAGE_DESCRIPTOR = {
        key: "navigation.on_this_page",
        message: "On this page",
        comment: "Compact UI label in site navigation, mobile drawer controls, or accessibility labels. Keep it brief and unambiguous.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_PAGE_NOT_FOUND_DESCRIPTION_DESCRIPTOR = {
        key: "navigation.page_not_found.description",
        message: "This page doesn't exist. But there's plenty more to explore.",
        comment: "Body copy on the 404 page. Keep the tone helpful and concise while making clear the page was not found.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_PAGE_NOT_FOUND_TITLE_DESCRIPTOR = {
        key: "navigation.page_not_found.title",
        message: "Page not found",
        comment: "Short UI label or heading on the 404 page. Keep the tone helpful and concise while making clear the page was not found.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_PRESS_DOWNLOAD_ASSETS_INTRO_DESCRIPTOR = {
        key: "navigation.press.download_assets_intro",
        message: "Download our logos, learn about our brand colors, and get in touch with our press team.",
        comment: "Introductory body copy on the press/brand-assets page. Keep wording professional and clear for journalists, partners, and brand-asset users.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_PRESS_DOWNLOAD_FLUXER_ASSETS_DESCRIPTOR = {
        key: "navigation.press.download_fluxer_assets",
        message: "Download {product_name} logos, brand assets, and get in touch with our press team",
        comment: "Press-page heading or summary line for brand asset downloads. Preserve {product_name} exactly; keep wording professional for journalists and partners. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const NAVIGATION_PRESS_PRESS_AND_BRAND_ASSETS_DESCRIPTOR = {
        key: "navigation.press.press_and_brand_assets",
        message: "Press and brand assets",
        comment: "Compact UI label on the press/brand-assets page. Keep wording professional and clear for journalists or partners.",
    };
);
