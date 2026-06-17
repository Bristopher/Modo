// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createAuthHarness, createTestAccount} from './AuthTestUtils';
import {
	createTotpSecret,
	decodeBase64URL,
	generateTotpCode,
	type WebAuthnRegistrationOptions,
} from './WebAuthnTestUtils';

describe('WebAuthn registration user handle', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createAuthHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	afterAll(async () => {
		await harness?.shutdown();
	});
	it('ensures registration options carry the stable user identifier', async () => {
		const account = await createTestAccount(harness);
		const secret = createTotpSecret();
		await createBuilder(harness, account.token)
			.post('/users/@me/mfa/totp/enable')
			.body({secret, code: generateTotpCode(secret), password: account.password})
			.expect(200)
			.execute();
		const regOptions = await createBuilder<WebAuthnRegistrationOptions>(harness, account.token)
			.post('/users/@me/mfa/webauthn/credentials/registration-options')
			.body({mfa_method: 'totp', mfa_code: generateTotpCode(secret)})
			.execute();
		const handle = decodeBase64URL(regOptions.user.id).toString();
		expect(handle).toBe(account.userId);
	});
});
