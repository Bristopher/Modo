// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LocaleCode} from '@fluxer/constants/src/Locales';
import {
	DEFAULT_LOCALE,
	findLocaleByLanguagePrefix,
	getLocaleByCode,
	getPreferredLocaleForLanguageCode,
	normalizeLocaleCode,
} from '@pkgs/locale/src/catalog/LocaleCatalog';

interface AcceptLanguagePreference {
	locale: string;
	quality: number;
}

export function resolveLocaleFromAcceptLanguageHeader(acceptLanguageHeader: string | null | undefined): LocaleCode {
	if (!acceptLanguageHeader) {
		return DEFAULT_LOCALE;
	}
	try {
		const preferences = parseAcceptLanguagePreferences(acceptLanguageHeader);
		const exactLocaleMatch = findExactLocaleMatch(preferences);
		if (exactLocaleMatch) {
			return exactLocaleMatch;
		}
		const languageFallbackMatch = findLanguageFallbackMatch(preferences);
		if (languageFallbackMatch) {
			return languageFallbackMatch;
		}
		return DEFAULT_LOCALE;
	} catch {
		return DEFAULT_LOCALE;
	}
}

function parseAcceptLanguagePreferences(acceptLanguageHeader: string): Array<AcceptLanguagePreference> {
	return acceptLanguageHeader
		.split(',')
		.map((languageToken) => parseLanguagePreference(languageToken))
		.sort((a, b) => b.quality - a.quality);
}

function parseLanguagePreference(languageToken: string): AcceptLanguagePreference {
	const parts = languageToken.trim().split(';');
	const locale = parts[0].trim();
	const quality = parseQualityValue(parts[1]);
	return {
		locale,
		quality,
	};
}

function parseQualityValue(rawQuality: string | undefined): number {
	const qualityMatch = rawQuality?.match(/q=([\d.]+)/);
	if (!qualityMatch) {
		return 1.0;
	}
	const parsedQuality = Number.parseFloat(qualityMatch[1]);
	if (!Number.isFinite(parsedQuality)) {
		return 1.0;
	}
	return parsedQuality;
}

function findExactLocaleMatch(preferences: ReadonlyArray<AcceptLanguagePreference>): LocaleCode | null {
	for (const {locale} of preferences) {
		const exactMatch = getLocaleByCode(locale);
		if (exactMatch) {
			return exactMatch;
		}
	}
	return null;
}

function findLanguageFallbackMatch(preferences: ReadonlyArray<AcceptLanguagePreference>): LocaleCode | null {
	for (const {locale} of preferences) {
		const languageCode = getLanguageCode(locale);
		const preferredFallbackLocale = getPreferredLocaleForLanguageCode(languageCode);
		if (preferredFallbackLocale) {
			return preferredFallbackLocale;
		}
		const prefixMatchLocale = findLocaleByLanguagePrefix(languageCode);
		if (prefixMatchLocale) {
			return prefixMatchLocale;
		}
	}
	return null;
}

function getLanguageCode(locale: string): string {
	const normalizedLocale = normalizeLocaleCode(locale);
	const [languageCode] = normalizedLocale.split('-');
	return languageCode ?? '';
}
