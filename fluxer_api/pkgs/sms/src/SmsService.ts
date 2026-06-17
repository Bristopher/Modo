// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ISmsService} from '@pkgs/sms/src/ISmsService';
import type {PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import {UnavailableSmsProvider} from '@pkgs/sms/src/providers/UnavailableSmsProvider';
import type {SmsVerificationStartOptions, SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';

export class SmsService implements ISmsService {
	private readonly provider: ISmsProvider;

	constructor(provider: ISmsProvider = new UnavailableSmsProvider()) {
		this.provider = provider;
	}

	async startVerification(phone: string): Promise<void> {
		await this.provider.startVerification(phone);
	}

	async startVerificationWithResult(
		phone: string,
		options?: SmsVerificationStartOptions,
	): Promise<SmsVerificationStartResult> {
		return this.provider.startVerificationWithResult(phone, options);
	}

	async checkVerification(phone: string, code: string): Promise<boolean> {
		return this.provider.checkVerification(phone, code);
	}

	async lookupPhone(phone: string): Promise<PhoneLookupResult | null> {
		return this.provider.lookupPhone(phone);
	}
}
