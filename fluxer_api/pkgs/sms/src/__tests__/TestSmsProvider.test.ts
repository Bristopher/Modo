// SPDX-License-Identifier: AGPL-3.0-or-later

import {createMockLogger} from '@fluxer/logger/src/mock';
import {TestSmsProvider} from '@pkgs/sms/src/providers/TestSmsProvider';
import {describe, expect, it} from 'vitest';

describe('TestSmsProvider', () => {
	describe('startVerification', () => {
		it('completes without error', async () => {
			const logger = createMockLogger();
			const provider = new TestSmsProvider({logger});
			await expect(provider.startVerification('+15551234567')).resolves.toBeUndefined();
		});
		it('supports different phone number formats', async () => {
			const logger = createMockLogger();
			const provider = new TestSmsProvider({logger});
			await expect(provider.startVerification('+14155552671')).resolves.toBeUndefined();
			await expect(provider.startVerification('+447911123456')).resolves.toBeUndefined();
			await expect(provider.startVerification('+81312345678')).resolves.toBeUndefined();
		});
	});
	describe('checkVerification', () => {
		it('returns true for the default valid code', async () => {
			const logger = createMockLogger();
			const provider = new TestSmsProvider({logger});
			await provider.startVerification('+15551234567');
			const result = await provider.checkVerification('+15551234567', '123456');
			expect(result).toBe(true);
		});
		it('returns false for invalid codes', async () => {
			const logger = createMockLogger();
			const provider = new TestSmsProvider({logger});
			await provider.startVerification('+15551234567');
			expect(await provider.checkVerification('+15551234567', '000000')).toBe(false);
			expect(await provider.checkVerification('+15551234567', '654321')).toBe(false);
			expect(await provider.checkVerification('+15551234567', 'abcdef')).toBe(false);
			expect(await provider.checkVerification('+15551234567', '')).toBe(false);
		});
		it('supports custom verification code overrides', async () => {
			const logger = createMockLogger();
			const provider = new TestSmsProvider({logger, verificationCode: '654321'});
			expect(await provider.checkVerification('+15551111111', '123456')).toBe(false);
			expect(await provider.checkVerification('+15551111111', '654321')).toBe(true);
		});
	});
});
