// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Context;
use gettext::Catalog;
use std::collections::BTreeMap;
use std::io::Cursor;

use crate::invariant_text::{BRAND_PLACEHOLDERS, PRODUCT_NAME};

include!(concat!(env!("OUT_DIR"), "/i18n/generated.rs"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarketingMessageDescriptor {
    key: &'static str,
    message: &'static str,
    comment: &'static str,
}

impl MarketingMessageDescriptor {
    pub const fn new(key: &'static str, message: &'static str, comment: &'static str) -> Self {
        Self {
            key,
            message,
            comment,
        }
    }

    pub const fn key(self) -> &'static str {
        self.key
    }

    pub const fn message(self) -> &'static str {
        self.message
    }

    pub const fn comment(self) -> &'static str {
        self.comment
    }
}

#[macro_export]
macro_rules! marketing_message {
    (
		pub const $name:ident = {
			key: $key:literal,
			message: $message:expr,
			comment: $comment:literal $(,)?
		};
	) => {
        pub const $name: $crate::i18n::MarketingMessageDescriptor =
            $crate::i18n::MarketingMessageDescriptor::new($key, $message, $comment);
    };
}

pub mod descriptors;

#[derive(Clone)]
pub struct MarketingI18n {
    catalogs: BTreeMap<Locale, Catalog>,
    defaults: MarketingDefaults,
}

#[derive(Clone)]
pub struct MarketingDefaults {
    pub l10n_email: &'static str,
    pub partners_email: &'static str,
    pub premium_tier_name: &'static str,
    pub product_name: &'static str,
    pub social_handle: &'static str,
}

impl Default for MarketingDefaults {
    fn default() -> Self {
        Self {
            l10n_email: "i18n@fluxer.app",
            partners_email: "partners@fluxer.app",
            premium_tier_name: "Plutonium",
            product_name: PRODUCT_NAME,
            social_handle: "fluxer.app",
        }
    }
}

impl MarketingI18n {
    pub fn new() -> anyhow::Result<Self> {
        let mut catalogs = BTreeMap::new();
        for locale in Locale::ALL {
            let catalog =
                Catalog::parse(Cursor::new(locale.catalog_bytes())).with_context(|| {
                    format!("failed to parse embedded gettext catalog {}", locale.code())
                })?;
            catalogs.insert(*locale, catalog);
        }
        Ok(Self {
            catalogs,
            defaults: MarketingDefaults::default(),
        })
    }

    pub fn text(&self, locale: Locale, descriptor: MarketingMessageDescriptor) -> String {
        self.text_with(locale, descriptor, &[])
    }

    pub fn text_with(
        &self,
        locale: Locale,
        descriptor: MarketingMessageDescriptor,
        vars: &[(&str, &str)],
    ) -> String {
        let template = self.localized_template(locale, descriptor);
        self.interpolate(descriptor, template, vars)
    }

    pub fn template(&self, locale: Locale, descriptor: MarketingMessageDescriptor) -> String {
        self.interpolate_defaults(self.localized_template(locale, descriptor))
    }

    pub fn locale_from_code(&self, code: &str) -> Option<Locale> {
        normalize_locale_code(code).and_then(|normalized| {
            Locale::ALL.iter().copied().find(|locale| {
                normalize_locale_code(locale.code()).as_deref() == Some(normalized.as_str())
            })
        })
    }

    pub fn preferred_locale_for_language(&self, language: &str) -> Option<Locale> {
        match normalize_locale_code(language).as_deref()? {
            "en" => Some(Locale::EnUs),
            "es" => Some(Locale::EsEs),
            "pt" => Some(Locale::PtBr),
            "zh" => Some(Locale::ZhCn),
            "sv" => Some(Locale::SvSe),
            normalized => Locale::ALL.iter().copied().find(|locale| {
                normalize_locale_code(locale.code())
                    .as_deref()
                    .map(|code| code.split('-').next().unwrap_or(code) == normalized)
                    .unwrap_or(false)
            }),
        }
    }

    fn interpolate(
        &self,
        descriptor: MarketingMessageDescriptor,
        template: &str,
        vars: &[(&str, &str)],
    ) -> String {
        let mut output = self.interpolate_defaults(template);
        for (name, value) in vars {
            output = output.replace(&format!("{{{name}}}"), value);
        }
        for placeholder in extract_placeholders(&output) {
            tracing::warn!(
                key = descriptor.key,
                %placeholder,
                "marketing translation rendered with an unresolved placeholder",
            );
        }
        output
    }

    fn localized_template(&self, locale: Locale, descriptor: MarketingMessageDescriptor) -> &str {
        let source = descriptor.message;
        if locale == Locale::EnUs {
            source
        } else {
            self.catalogs
                .get(&locale)
                .map(|catalog| catalog.pgettext(descriptor.key, source))
                .unwrap_or(source)
        }
    }

    fn interpolate_defaults(&self, template: &str) -> String {
        let mut output = template.to_owned();
        for (name, value) in self.default_pairs() {
            output = output.replace(&format!("{{{name}}}"), value);
        }
        output
    }

    fn default_pairs(&self) -> Vec<(&'static str, &'static str)> {
        let mut pairs = vec![
            ("l10n_email", self.defaults.l10n_email),
            ("partners_email", self.defaults.partners_email),
            ("premium_tier_name", self.defaults.premium_tier_name),
            ("product_name", self.defaults.product_name),
            ("social_handle", self.defaults.social_handle),
        ];
        pairs.extend_from_slice(BRAND_PLACEHOLDERS);
        pairs
    }
}

pub fn normalize_locale_code(code: &str) -> Option<String> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.replace('_', "-").to_ascii_lowercase())
}

fn extract_placeholders(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'{' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let Some(end_offset) = input[start..].find('}') else {
            index += 1;
            continue;
        };
        let end = start + end_offset;
        let candidate = &input[start..end];
        if is_placeholder_name(candidate) {
            result.push(candidate.to_owned());
        }
        index = end + 1;
    }
    result
}

fn is_placeholder_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
