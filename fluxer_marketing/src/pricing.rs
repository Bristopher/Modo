// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Currency {
    Usd,
    Eur,
    Brl,
    Inr,
    Pln,
    Try,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PricingTier {
    Monthly,
    Yearly,
    Operator,
}

const EEA_COUNTRIES: &[&str] = &[
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR", "HU", "IE", "IT", "LV",
    "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK", "SI", "ES", "SE", "IS", "LI", "NO",
];

impl Currency {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Eur => "EUR",
            Self::Brl => "BRL",
            Self::Inr => "INR",
            Self::Pln => "PLN",
            Self::Try => "TRY",
        }
    }

    pub const fn donation_code(self) -> &'static str {
        match self {
            Self::Usd => "usd",
            Self::Eur => "eur",
            Self::Brl => "brl",
            Self::Inr => "inr",
            Self::Pln => "pln",
            Self::Try => "try",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Usd => "$",
            Self::Eur => "€",
            Self::Brl => "R$",
            Self::Inr => "₹",
            Self::Pln => "zł",
            Self::Try => "₺",
        }
    }
}

pub fn get_currency(country_code: &str) -> Currency {
    match country_code.to_ascii_uppercase().as_str() {
        "BR" => Currency::Brl,
        "IN" => Currency::Inr,
        "PL" => Currency::Pln,
        "TR" => Currency::Try,
        code if is_eea_country(code) => Currency::Eur,
        _ => Currency::Usd,
    }
}

pub fn get_base_currency(country_code: &str) -> Currency {
    if is_eea_country(country_code) {
        Currency::Eur
    } else {
        Currency::Usd
    }
}

pub fn has_localized_pricing_choice(country_code: &str) -> bool {
    matches!(
        country_code.to_ascii_uppercase().as_str(),
        "BR" | "IN" | "PL" | "TR"
    )
}

pub fn is_eea_country(country_code: &str) -> bool {
    let upper = country_code.to_ascii_uppercase();
    EEA_COUNTRIES.contains(&upper.as_str())
}

pub fn get_price_minor(tier: PricingTier, currency: Currency) -> u32 {
    match (tier, currency) {
        (PricingTier::Monthly, Currency::Usd) => 499,
        (PricingTier::Monthly, Currency::Eur) => 499,
        (PricingTier::Monthly, Currency::Brl) => 2499,
        (PricingTier::Monthly, Currency::Inr) => 49_900,
        (PricingTier::Monthly, Currency::Pln) => 1799,
        (PricingTier::Monthly, Currency::Try) => 22_999,
        (PricingTier::Yearly, Currency::Usd) => 4999,
        (PricingTier::Yearly, Currency::Eur) => 4999,
        (PricingTier::Yearly, Currency::Brl) => 24_999,
        (PricingTier::Yearly, Currency::Inr) => 499_900,
        (PricingTier::Yearly, Currency::Pln) => 17_999,
        (PricingTier::Yearly, Currency::Try) => 229_999,
        (PricingTier::Operator, Currency::Usd) => 19_900,
        (PricingTier::Operator, Currency::Eur) => 19_900,
        (PricingTier::Operator, Currency::Brl) => 99_999,
        (PricingTier::Operator, Currency::Inr) => 1_899_900,
        (PricingTier::Operator, Currency::Pln) => 71_999,
        (PricingTier::Operator, Currency::Try) => 899_999,
    }
}

pub fn format_major_amount(amount: u32, currency: Currency) -> String {
    match currency {
        Currency::Usd | Currency::Eur | Currency::Inr | Currency::Try => {
            format!("{}{}", currency.symbol(), amount)
        }
        Currency::Brl | Currency::Pln => format!("{} {}", currency.symbol(), amount),
    }
}

pub fn format_price_minor(price_minor: u32, currency: Currency) -> String {
    let major = price_minor as f64 / 100.0;
    if price_minor.is_multiple_of(100) {
        format_major_amount(price_minor / 100, currency)
    } else {
        match currency {
            Currency::Usd | Currency::Eur | Currency::Inr | Currency::Try => {
                format!("{}{major:.2}", currency.symbol())
            }
            Currency::Brl | Currency::Pln => format!("{} {major:.2}", currency.symbol()),
        }
    }
}
