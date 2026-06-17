// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {checkGeoVsLocale} from '../adapters/LocaleGeoMatcher';

describe('checkGeoVsLocale', () => {
	it('matches en-US to US', async () => {
		const r = await checkGeoVsLocale({geoipCountryIso: 'US', registrationLocale: 'en-US', registrationTimezone: null});
		expect(r.localeGeoMatch).toBe(true);
		expect(r.mismatchDetected).toBe(false);
	});
	it('flags en-US from RU as a mismatch', async () => {
		const r = await checkGeoVsLocale({geoipCountryIso: 'RU', registrationLocale: 'en-US', registrationTimezone: null});
		expect(r.mismatchDetected).toBe(true);
	});
	it('matches ja-JP to JP', async () => {
		const r = await checkGeoVsLocale({geoipCountryIso: 'JP', registrationLocale: 'ja-JP', registrationTimezone: null});
		expect(r.localeGeoMatch).toBe(true);
	});
	it('flags ja from non-JP as language-only mismatch', async () => {
		const r = await checkGeoVsLocale({geoipCountryIso: 'BR', registrationLocale: 'ja', registrationTimezone: null});
		expect(r.mismatchDetected).toBe(true);
	});
	it('returns no mismatch when one input is missing', async () => {
		const r = await checkGeoVsLocale({geoipCountryIso: null, registrationLocale: 'en-US', registrationTimezone: null});
		expect(r.mismatchDetected).toBe(false);
	});
});
