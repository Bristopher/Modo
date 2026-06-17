// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const VOICE_REGIONS_MAP_HEADING_DESCRIPTOR = {
        key: "voice_regions.map_heading",
        message: "Voice regions",
        comment: "Section heading above the world map of voice regions. Plain, sentence case.",
    };
);

crate::marketing_message!(
    pub const VOICE_REGIONS_MAP_INTRO_DESCRIPTOR = {
        key: "voice_regions.map_intro",
        message: "Sixteen voice regions across six continents. Voice and video calls connect through the region closest to you.",
        comment: "Paragraph under the map heading. State facts; no marketing language. Sentence case.",
    };
);

crate::marketing_message!(
    pub const VOICE_REGIONS_MAP_LEGEND_DESCRIPTOR = {
        key: "voice_regions.map_legend",
        message: "Each dot is a {product_name} voice region.",
        comment: "Accessible legend / alt-style sentence next to the map. Sentence case, short. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const VOICE_REGIONS_LANGUAGES_HEADING_DESCRIPTOR = {
        key: "voice_regions.languages_heading",
        message: "Languages",
        comment: "Section heading above the list of supported languages. Sentence case; one short noun.",
    };
);

crate::marketing_message!(
    pub const VOICE_REGIONS_LANGUAGES_INTRO_DESCRIPTOR = {
        key: "voice_regions.languages_intro",
        message: "{product_name} is available in over thirty languages. To help translate {product_name} into your native language, write to {email}.",
        comment: "Paragraph under the languages heading. Includes an {email} placeholder for the localization contact. Plain and factual.",
    };
);
