// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LocaleCode} from '@fluxer/constants/src/Locales';
import {
	getLocaleByCode,
	getLocaleDisplayName,
	getLocaleFlagCode,
	getLocaleLanguageCode,
} from '@pkgs/locale/src/catalog/LocaleCatalog';
import {resolveLocaleFromAcceptLanguageHeader} from '@pkgs/locale/src/resolution/AcceptLanguageNegotiation';

interface LocaleMetadata {
	code: LocaleCode;
	languageCode: string;
	name: string;
	flagCode: string;
}

export function getLocaleMetadata(locale: LocaleCode): LocaleMetadata {
	return {
		code: locale,
		languageCode: getLocaleLanguageCode(locale),
		name: getLocaleDisplayName(locale),
		flagCode: getLocaleFlagCode(locale),
	};
}

export function getLocaleName(locale: LocaleCode): string {
	return getLocaleDisplayName(locale);
}

export function getFlagCode(locale: LocaleCode): string {
	return getLocaleFlagCode(locale);
}

export function getLocaleFromCode(code: string): LocaleCode | null {
	return getLocaleByCode(code);
}

export function getLocaleCode(locale: LocaleCode): string {
	return getLocaleLanguageCode(locale);
}

export function parseAcceptLanguage(acceptLanguageHeader: string | null | undefined): LocaleCode {
	return resolveLocaleFromAcceptLanguageHeader(acceptLanguageHeader);
}
