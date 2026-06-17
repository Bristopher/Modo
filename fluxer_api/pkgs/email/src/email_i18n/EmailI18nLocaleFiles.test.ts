// SPDX-License-Identifier: AGPL-3.0-or-later

import {getEmailTemplate, resetEmailI18n} from '@pkgs/email/src/email_i18n/EmailI18n';
import {EMAIL_I18N_LOCALE_MESSAGES} from '@pkgs/email/src/email_i18n/EmailI18nLocales';
import {EMAIL_I18N_MESSAGES} from '@pkgs/email/src/email_i18n/EmailI18nMessages';
import {afterEach, describe, expect, it} from 'vitest';

const LOCALES = Object.keys(EMAIL_I18N_LOCALE_MESSAGES) as Array<keyof typeof EMAIL_I18N_LOCALE_MESSAGES>;

describe('EmailI18n locale files', () => {
	afterEach(() => {
		resetEmailI18n();
	});
	it.each(LOCALES)('%s loads without module errors', (locale) => {
		const template = getEmailTemplate('email_verification', locale, {
			username: 'testuser',
			verifyUrl: 'https://example.com/verify',
		});
		expect(template.ok).toBe(true);
	});
	it.each(LOCALES)('%s has the same translation keys as the source catalog', (locale) => {
		const messagesKeys = Object.keys(EMAIL_I18N_MESSAGES).sort();
		const localeKeys = Object.keys(EMAIL_I18N_LOCALE_MESSAGES[locale]).sort();
		expect(localeKeys).toEqual(messagesKeys);
	});
});
