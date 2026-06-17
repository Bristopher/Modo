// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{LinkReplacement, flag_svg, locale_native_name, message_with_links, tr};
use crate::{
    i18n::{Locale, MarketingI18n, descriptors::*},
    request_context::RequestContext,
};
use maud::{Markup, html};

#[derive(Clone, Copy)]
struct VoiceRegionPin {
    local_name: &'static str,
    iata: &'static str,
    flag_code: &'static str,
    lat: f64,
    lon: f64,
}

const VOICE_REGION_PINS: &[VoiceRegionPin] = &[
    VoiceRegionPin {
        local_name: "Seattle",
        iata: "SEA",
        flag_code: "1f1fa-1f1f8",
        lat: 47.6038,
        lon: -122.33,
    },
    VoiceRegionPin {
        local_name: "Los Angeles",
        iata: "LAX",
        flag_code: "1f1fa-1f1f8",
        lat: 34.0549,
        lon: -118.2426,
    },
    VoiceRegionPin {
        local_name: "Newark",
        iata: "EWR",
        flag_code: "1f1fa-1f1f8",
        lat: 40.5455,
        lon: -74.4607,
    },
    VoiceRegionPin {
        local_name: "Atlanta",
        iata: "ATL",
        flag_code: "1f1fa-1f1f8",
        lat: 33.7488,
        lon: -84.39,
    },
    VoiceRegionPin {
        local_name: "Dallas",
        iata: "DFW",
        flag_code: "1f1fa-1f1f8",
        lat: 32.7767,
        lon: -96.7971,
    },
    VoiceRegionPin {
        local_name: "Warszawa",
        iata: "WAW",
        flag_code: "1f1f5-1f1f1",
        lat: 52.2297,
        lon: 21.0122,
    },
    VoiceRegionPin {
        local_name: "Frankfurt am Main",
        iata: "FRA",
        flag_code: "1f1e9-1f1ea",
        lat: 50.11,
        lon: 8.68,
    },
    VoiceRegionPin {
        local_name: "Stockholm",
        iata: "ARN",
        flag_code: "1f1f8-1f1ea",
        lat: 59.3327,
        lon: 18.0656,
    },
    VoiceRegionPin {
        local_name: "Madrid",
        iata: "MAD",
        flag_code: "1f1ea-1f1f8",
        lat: 40.4169,
        lon: -3.7033,
    },
    VoiceRegionPin {
        local_name: "Singapore",
        iata: "SIN",
        flag_code: "1f1f8-1f1ec",
        lat: 1.3649,
        lon: 103.8228,
    },
    VoiceRegionPin {
        local_name: "मुंबई",
        iata: "BOM",
        flag_code: "1f1ee-1f1f3",
        lat: 18.9582,
        lon: 72.8321,
    },
    VoiceRegionPin {
        local_name: "서울",
        iata: "ICN",
        flag_code: "1f1f0-1f1f7",
        lat: 37.5503,
        lon: 126.9971,
    },
    VoiceRegionPin {
        local_name: "São Paulo",
        iata: "GRU",
        flag_code: "1f1e7-1f1f7",
        lat: -23.5558,
        lon: -46.6396,
    },
    VoiceRegionPin {
        local_name: "Santiago",
        iata: "SCL",
        flag_code: "1f1e8-1f1f1",
        lat: -33.4489,
        lon: -70.6693,
    },
    VoiceRegionPin {
        local_name: "Johannesburg",
        iata: "JNB",
        flag_code: "1f1ff-1f1e6",
        lat: -26.2044,
        lon: 28.0455,
    },
    VoiceRegionPin {
        local_name: "Sydney",
        iata: "SYD",
        flag_code: "1f1e6-1f1fa",
        lat: -33.8623,
        lon: 151.2077,
    },
];

fn project_to_percent(lat: f64, lon: f64) -> (f64, f64) {
    let x = (lon + 180.0) / 360.0 * 100.0;
    let y = (90.0 - lat) / 180.0 * 100.0;
    (x, y)
}

fn flag_image(ctx: &RequestContext, flag_code: &str, class_name: &str) -> Markup {
    let flag_url = voice_region_flag_href(ctx, flag_code);
    html! {
        img src=(flag_url) alt="" aria-hidden="true" loading="eager" decoding="async" class=(class_name);
    }
}

pub(crate) fn voice_region_flag_preload_hrefs(ctx: &RequestContext) -> Vec<String> {
    let mut hrefs = Vec::new();
    for pin in VOICE_REGION_PINS {
        let href = voice_region_flag_href(ctx, pin.flag_code);
        if !hrefs.iter().any(|existing| existing == &href) {
            hrefs.push(href);
        }
    }
    hrefs
}

fn voice_region_flag_href(ctx: &RequestContext, flag_code: &str) -> String {
    ctx.href(&format!("/static/voice-region-flags/{flag_code}.svg"))
}

fn voice_regions_map(ctx: &RequestContext, legend: &str) -> Markup {
    html! {
        div class="relative mx-auto w-full rounded-2xl border border-gray-200 bg-gray-50"
            style="aspect-ratio: 2 / 1;"
            role="group"
            aria-label=(legend)
        {
            div class="absolute inset-0 overflow-hidden rounded-2xl" {
                img
                    src=(ctx.href("/static/world-map-equirectangular.svg"))
                    alt=""
                    aria-hidden="true"
                    loading="lazy"
                    decoding="async"
                    class="pointer-events-none absolute inset-0 h-full w-full object-cover opacity-20";
            }
            @for pin in VOICE_REGION_PINS {
                @let (x, y) = project_to_percent(pin.lat, pin.lon);
                @let label = format!("{} ({})", pin.local_name, pin.iata);
                @let pin_id = format!("voice-region-pin-{}", pin.iata.to_ascii_lowercase());
                @let tooltip_id = format!("{pin_id}-tooltip");
                div
                    class="voice-region-pin absolute h-6 w-6 rounded-full"
                    style=(format!("left: {x:.3}%; top: {y:.3}%; transform: translate(-50%, -50%);"))
                {
                    button id=(pin_id.as_str()) type="button" class="voice-region-pin-target" aria-label=(label) aria-describedby=(tooltip_id.as_str()) {
                        span class="absolute left-1/2 top-1/2 h-2.5 w-2.5 -translate-x-1/2 -translate-y-1/2" aria-hidden="true" {
                            span class="voice-region-pin-dot block h-full w-full rounded-full bg-[#4641D9] outline outline-2 outline-white" {}
                        }
                    }
                    span id=(tooltip_id.as_str()) role="tooltip" class="voice-region-tooltip pointer-events-none absolute left-1/2 bottom-full mb-2 -translate-x-1/2 items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 text-gray-900 shadow-md" {
                        (flag_image(ctx, pin.flag_code, "h-5 w-5 shrink-0 rounded-sm"))
                        span class="voice-region-tooltip-label font-semibold text-sm" { (pin.local_name) }
                    }
                }
            }
        }
    }
}

pub fn voice_regions_section(i18n: &MarketingI18n, ctx: &RequestContext) -> Markup {
    let legend = tr(i18n, ctx, VOICE_REGIONS_MAP_LEGEND_DESCRIPTOR);
    html! {
        section class="bg-white px-6 py-16 sm:px-8 md:px-12 md:py-24 lg:px-16 xl:px-20" {
            div class="mx-auto max-w-7xl" {
                div class="mb-12 text-center md:mb-16" {
                    h2 class="display mb-6 text-4xl text-black md:mb-8 md:text-5xl lg:text-6xl" {
                        (tr(i18n, ctx, VOICE_REGIONS_MAP_HEADING_DESCRIPTOR))
                    }
                    p class="lead mx-auto max-w-3xl text-gray-700" {
                        (tr(i18n, ctx, VOICE_REGIONS_MAP_INTRO_DESCRIPTOR))
                    }
                }
                (voice_regions_map(ctx, &legend))
            }
        }
    }
}

fn language_tile(ctx: &RequestContext, locale: Locale) -> Markup {
    html! {
        li class="inline-flex h-11 items-center gap-2.5 rounded-full border border-gray-200 bg-white px-4 text-gray-900" {
            (flag_svg(ctx, locale, "h-5 w-5 shrink-0 rounded-sm"))
            span class="whitespace-nowrap font-semibold text-sm" {
                (locale_native_name(locale))
            }
        }
    }
}

pub fn languages_section(i18n: &MarketingI18n, ctx: &RequestContext) -> Markup {
    let intro_template = i18n.template(ctx.locale, VOICE_REGIONS_LANGUAGES_INTRO_DESCRIPTOR);
    html! {
        section class="bg-gray-50 px-6 py-16 sm:px-8 md:px-12 md:py-24 lg:px-16 xl:px-20" {
            div class="mx-auto max-w-7xl" {
                div class="mb-12 text-center md:mb-16" {
                    h2 class="display mb-6 text-4xl text-black md:mb-8 md:text-5xl lg:text-6xl" {
                        (tr(i18n, ctx, VOICE_REGIONS_LANGUAGES_HEADING_DESCRIPTOR))
                    }
                    p class="lead mx-auto max-w-3xl text-gray-700" {
                        (message_with_links(&intro_template, &[
                            LinkReplacement {
                                variable: "email",
                                text: "i18n@fluxer.app",
                                href: "mailto:i18n@fluxer.app",
                                class: "text-[#4641D9] underline decoration-[#4641D9]/40 hover:decoration-[#4641D9]",
                            },
                        ]))
                    }
                }
                ul class="mx-auto flex max-w-6xl flex-wrap justify-center gap-3" {
                    @for locale in Locale::ALL {
                        (language_tile(ctx, *locale))
                    }
                }
            }
        }
    }
}
