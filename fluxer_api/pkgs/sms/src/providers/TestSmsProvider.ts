// SPDX-License-Identifier: AGPL-3.0-or-later

import {SMS_TEST_VERIFICATION_CODE} from '@fluxer/constants/src/SmsVerificationConstants';
import {createLogger} from '@fluxer/logger/src/Logger';
import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import {SMS_VERIFICATION_START_SMS_RESULT, type SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';
import {maskPhoneNumber} from '@pkgs/sms/src/SmsVerificationUtils';

interface TestSmsProviderOptions {
	logger?: LoggerInterface;
	verificationCode?: string;
}

export class TestSmsProvider implements ISmsProvider {
	private readonly logger: LoggerInterface;
	private readonly verificationCode: string;

	constructor({logger, verificationCode}: TestSmsProviderOptions = {}) {
		this.logger = logger ?? createLogger('@pkgs/sms/src', {environment: 'test'});
		this.verificationCode = verificationCode ?? SMS_TEST_VERIFICATION_CODE;
	}

	async startVerification(phone: string): Promise<void> {
		this.logger.info(
			`[TestSmsProvider] Mock verification started for ${maskPhoneNumber(phone)}. Use code: ${this.verificationCode}`,
		);
	}

	async startVerificationWithResult(phone: string): Promise<SmsVerificationStartResult> {
		await this.startVerification(phone);
		return SMS_VERIFICATION_START_SMS_RESULT;
	}

	async checkVerification(phone: string, code: string): Promise<boolean> {
		const isValid = code === this.verificationCode;
		this.logger.info(
			`[TestSmsProvider] Mock verification check for ${maskPhoneNumber(phone)} with code ${code}: ${isValid ? 'APPROVED' : 'REJECTED'}`,
		);
		return isValid;
	}

	async lookupPhone(phone: string): Promise<PhoneLookupResult | null> {
		this.logger.info(`[TestSmsProvider] Mock lookup for ${maskPhoneNumber(phone)} -> valid mobile`);
		return {
			valid: true,
			lineType: 'mobile',
			countryCode: null,
			carrierName: 'Test Carrier',
			smsPumpingRiskScore: null,
		};
	}
}
