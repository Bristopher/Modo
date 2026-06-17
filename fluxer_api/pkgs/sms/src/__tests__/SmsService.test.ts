// SPDX-License-Identifier: AGPL-3.0-or-later

import {APIErrorCodes} from '@fluxer/constants/src/ApiErrorCodes';
import {SmsVerificationUnavailableError} from '@fluxer/errors/src/domains/auth/SmsVerificationUnavailableError';
import type {PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import {UnavailableSmsProvider} from '@pkgs/sms/src/providers/UnavailableSmsProvider';
import {SmsService} from '@pkgs/sms/src/SmsService';
import {SMS_VERIFICATION_START_SMS_RESULT, type SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';
import {describe, expect, it} from 'vitest';

function createInMemoryProvider(): ISmsProvider & {
	verifications: Map<string, string>;
	startedVerifications: Array<string>;
} {
	const verifications = new Map<string, string>();
	const startedVerifications: Array<string> = [];
	return {
		verifications,
		startedVerifications,
		async startVerification(phone: string): Promise<void> {
			startedVerifications.push(phone);
			verifications.set(phone, '123456');
		},
		async startVerificationWithResult(phone: string): Promise<SmsVerificationStartResult> {
			startedVerifications.push(phone);
			verifications.set(phone, '123456');
			return SMS_VERIFICATION_START_SMS_RESULT;
		},
		async checkVerification(phone: string, code: string): Promise<boolean> {
			const storedCode = verifications.get(phone);
			if (storedCode === code) {
				verifications.delete(phone);
				return true;
			}
			return false;
		},
		async lookupPhone(_phone: string): Promise<PhoneLookupResult | null> {
			return null;
		},
	};
}

describe('SmsService', () => {
	describe('with provider', () => {
		it('starts verification through provider', async () => {
			const provider = createInMemoryProvider();
			const service = new SmsService(provider);
			await service.startVerification('+15551234567');
			expect(provider.startedVerifications).toContain('+15551234567');
			expect(provider.verifications.has('+15551234567')).toBe(true);
		});
		it('checks verification through provider and returns true for valid code', async () => {
			const provider = createInMemoryProvider();
			const service = new SmsService(provider);
			await service.startVerification('+15551234567');
			const code = provider.verifications.get('+15551234567') ?? '';
			const result = await service.checkVerification('+15551234567', code);
			expect(result).toBe(true);
		});
		it('checks verification through provider and returns false for invalid code', async () => {
			const provider = createInMemoryProvider();
			const service = new SmsService(provider);
			await service.startVerification('+15551234567');
			const result = await service.checkVerification('+15551234567', 'wrong-code');
			expect(result).toBe(false);
		});
		it('returns false for verification check on non-existent phone', async () => {
			const provider = createInMemoryProvider();
			const service = new SmsService(provider);
			const result = await service.checkVerification('+15559999999', '123456');
			expect(result).toBe(false);
		});
	});
	describe('with unavailable provider', () => {
		it('silently completes startVerification when provider is unavailable', async () => {
			const service = new SmsService(new UnavailableSmsProvider());
			await expect(service.startVerification('+15551234567')).resolves.toBeUndefined();
		});
		it('throws SmsVerificationUnavailableError when checking verification', async () => {
			const service = new SmsService(new UnavailableSmsProvider());
			await expect(service.checkVerification('+15551234567', '123456')).rejects.toThrow(
				SmsVerificationUnavailableError,
			);
		});
		it('defaults to unavailable provider when no provider is injected', async () => {
			const service = new SmsService();
			await expect(service.checkVerification('+15551234567', '123456')).rejects.toThrow(
				SmsVerificationUnavailableError,
			);
		});
		it('exposes the correct api error code when checking verification', async () => {
			const service = new SmsService(new UnavailableSmsProvider());
			await expect(service.checkVerification('+15551234567', '123456')).rejects.toMatchObject({
				code: APIErrorCodes.SMS_VERIFICATION_UNAVAILABLE,
				message: APIErrorCodes.SMS_VERIFICATION_UNAVAILABLE,
			});
		});
	});
});
