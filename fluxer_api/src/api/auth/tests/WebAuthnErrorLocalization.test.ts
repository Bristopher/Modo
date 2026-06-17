// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createAuthHarness, createTestAccount} from './AuthTestUtils';
import {
	createRegistrationResponse,
	createTotpSecret,
	createWebAuthnDevice,
	generateTotpCode,
	type WebAuthnRegistrationOptions,
} from './WebAuthnTestUtils';

describe('WebAuthn error localization', () => {
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
	it('ensures WebAuthn registration errors return localized messages', async () => {
		const account = await createTestAccount(harness);
		const device = createWebAuthnDevice();
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
		if (regOptions.rp.id) {
			device.rpId = regOptions.rp.id;
		}
		const badChallenge = `${regOptions.challenge}-tampered`;
		const registrationResponse = createRegistrationResponse(device, regOptions, 'Localized error');
		const errResp = await createBuilder<{
			code: string;
			message: string;
		}>(harness, account.token)
			.post('/users/@me/mfa/webauthn/credentials')
			.body({
				response: registrationResponse,
				challenge: badChallenge,
				name: 'Localized error',
				mfa_method: 'totp',
				mfa_code: generateTotpCode(secret),
			})
			.expect(400)
			.execute();
		expect(errResp.code).toBe('INVALID_WEBAUTHN_CREDENTIAL');
		expect(errResp.message).toBe('Failed to verify WebAuthn credential.');
	});
});
