// SPDX-License-Identifier: AGPL-3.0-or-later

import {SmsVerificationUnavailableError} from '@fluxer/errors/src/domains/auth/SmsVerificationUnavailableError';
import {createMockLogger} from '@fluxer/logger/src/mock';
import {createSmsProvider} from '@pkgs/sms/src/providers/SmsProviderFactory';
import {describe, expect, it} from 'vitest';

describe('createSmsProvider', () => {
	it('creates a test provider that accepts the configured code', async () => {
		const provider = createSmsProvider({
			mode: 'test',
			logger: createMockLogger(),
			verificationCode: '654321',
		});
		await expect(provider.startVerification('+15551234567')).resolves.toBeUndefined();
		await expect(provider.checkVerification('+15551234567', '654321')).resolves.toBe(true);
		await expect(provider.checkVerification('+15551234567', '123456')).resolves.toBe(false);
	});
	it('creates an unavailable provider that throws on verification checks', async () => {
		const provider = createSmsProvider({
			mode: 'unavailable',
			logger: createMockLogger(),
		});
		await expect(provider.startVerification('+15551234567')).resolves.toBeUndefined();
		await expect(provider.checkVerification('+15551234567', '123456')).rejects.toThrow(SmsVerificationUnavailableError);
	});
	it('creates a Twilio provider in twilio mode', async () => {
		const provider = createSmsProvider({
			mode: 'twilio',
			config: {
				accountSid: 'AC123',
				authToken: 'twilio-secret',
				verifyServiceSid: 'VA123',
			},
			logger: createMockLogger(),
			fetchFn: async () => new Response(JSON.stringify({status: 'pending'}), {status: 200}),
		});
		await expect(provider.startVerification('+15551234567')).resolves.toBeUndefined();
	});
});
