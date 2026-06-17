// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder, createBuilderWithoutAuth} from '../../test/TestRequestBuilder';
import {createAuthHarness, createTestAccount} from './AuthTestUtils';
import {
	createAuthenticationResponseWithoutUV,
	createRegistrationResponse,
	createTotpSecret,
	createWebAuthnDevice,
	generateTotpCode,
	type WebAuthnAuthenticationOptions,
	type WebAuthnRegistrationOptions,
} from './WebAuthnTestUtils';

describe('WebAuthn user verification required', () => {
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
	it('ensures WebAuthn authentication requires user verification (UV flag set)', async () => {
		const account = await createTestAccount(harness);
		const device = createWebAuthnDevice();
		const secret = createTotpSecret();
		await createBuilder(harness, account.token)
			.post('/users/@me/mfa/totp/enable')
			.body({secret, code: generateTotpCode(secret), password: account.password})
			.execute();
		const regOptions = await createBuilder<WebAuthnRegistrationOptions>(harness, account.token)
			.post('/users/@me/mfa/webauthn/credentials/registration-options')
			.body({mfa_method: 'totp', mfa_code: generateTotpCode(secret)})
			.execute();
		if (regOptions.rp.id) {
			device.rpId = regOptions.rp.id;
		}
		const registrationResponse = createRegistrationResponse(device, regOptions, 'UV required');
		await createBuilder(harness, account.token)
			.post('/users/@me/mfa/webauthn/credentials')
			.body({
				response: registrationResponse,
				challenge: regOptions.challenge,
				name: 'UV required',
				mfa_method: 'totp',
				mfa_code: generateTotpCode(secret),
			})
			.expect(204)
			.execute();
		const authOptions = await createBuilderWithoutAuth<WebAuthnAuthenticationOptions>(harness)
			.post('/auth/webauthn/authentication-options')
			.body(null)
			.execute();
		expect(authOptions.userVerification).toBe('required');
		if (authOptions.rpId) {
			device.rpId = authOptions.rpId;
		}
		const assertion = createAuthenticationResponseWithoutUV(device, authOptions);
		await createBuilderWithoutAuth(harness)
			.post('/auth/webauthn/authenticate')
			.body({
				response: assertion,
				challenge: authOptions.challenge,
			})
			.execute();
		expect(authOptions.userVerification).toBe('required');
	});
});
