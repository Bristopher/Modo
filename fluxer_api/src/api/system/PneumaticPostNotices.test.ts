// SPDX-License-Identifier: AGPL-3.0-or-later

import {AllLocales, Locales} from '@fluxer/constants/src/Locales';
import {describe, expect, test} from 'vitest';
import {
	PLUTONIUM_MOBILE_BETA_DISPATCH,
	resolvePlutoniumMobileBetaCorrectionBody,
	resolvePlutoniumMobileBetaDispatchBody,
} from './PneumaticPostNotices';

const TEST_USER_ID = '123456789012345678';

const FIXED_LATIN_PRODUCT_NAMES = [
	PLUTONIUM_MOBILE_BETA_DISPATCH.productName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.premiumProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.androidProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.iosProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.githubProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.testFlightProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.appleProductName,
	PLUTONIUM_MOBILE_BETA_DISPATCH.appStoreProductName,
];

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

describe('PneumaticPostNotices', () => {
	test.each(AllLocales)('renders the Plutonium mobile beta dispatch for %s', (locale) => {
		const body = resolvePlutoniumMobileBetaDispatchBody(locale, TEST_USER_ID);

		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.productName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.premiumProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.androidProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.iosProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.githubUrl);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.formUrl);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.testFlightProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.appleProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.appStoreProductName);
		expect(body).toContain(TEST_USER_ID);
		expect(body).not.toMatch(/\{\{[a-z0-9_]+}}/);
		expect(body.split('\n\n')).toHaveLength(8);
		expect(body).not.toMatch(/\]\([^)]*\)[A-Za-zÀ-ž]/u);

		for (const productName of FIXED_LATIN_PRODUCT_NAMES) {
			expect(body).not.toMatch(new RegExp(`${escapeRegExp(productName)}[A-Za-zÀ-ž]`, 'u'));
		}
	});

	test.each(AllLocales)('renders the Plutonium mobile beta correction for %s', (locale) => {
		const body = resolvePlutoniumMobileBetaCorrectionBody(locale, TEST_USER_ID);

		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.productName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.androidProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.iosProductName);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.formUrl);
		expect(body).toContain(PLUTONIUM_MOBILE_BETA_DISPATCH.appStoreProductName);
		expect(body).toContain(TEST_USER_ID);
		expect(body).not.toMatch(/\{\{[a-z0-9_]+}}/);
		expect(body.split('\n\n')).toHaveLength(5);
		expect(body).not.toMatch(/\]\([^)]*\)[A-Za-zÀ-ž]/u);
	});

	test('uses the English template for English locales and unsupported locales', () => {
		const english = resolvePlutoniumMobileBetaDispatchBody(Locales.EN_US, TEST_USER_ID);

		expect(resolvePlutoniumMobileBetaDispatchBody(Locales.EN_GB, TEST_USER_ID)).toBe(english);
		expect(resolvePlutoniumMobileBetaDispatchBody('not-a-locale', TEST_USER_ID)).toBe(english);
	});

	test.each(
		AllLocales.filter((locale) => locale !== Locales.EN_US && locale !== Locales.EN_GB),
	)('uses a localized template for %s', (locale) => {
		expect(resolvePlutoniumMobileBetaDispatchBody(locale, TEST_USER_ID)).not.toBe(
			resolvePlutoniumMobileBetaDispatchBody(Locales.EN_US, TEST_USER_ID),
		);
	});
});
