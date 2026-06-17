// SPDX-License-Identifier: AGPL-3.0-or-later

import {SmsVerificationUnavailableError} from '@fluxer/errors/src/domains/auth/SmsVerificationUnavailableError';
import type {PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import {SMS_VERIFICATION_START_SMS_RESULT, type SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';

export class UnavailableSmsProvider implements ISmsProvider {
	async startVerification(_phone: string): Promise<void> {
		return;
	}

	async startVerificationWithResult(_phone: string): Promise<SmsVerificationStartResult> {
		return SMS_VERIFICATION_START_SMS_RESULT;
	}

	async checkVerification(_phone: string, _code: string): Promise<boolean> {
		throw new SmsVerificationUnavailableError();
	}

	async lookupPhone(_phone: string): Promise<PhoneLookupResult | null> {
		return null;
	}
}
