// SPDX-License-Identifier: AGPL-3.0-or-later

import type {PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {SmsVerificationStartOptions, SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';

export interface ISmsService {
	startVerification(phone: string): Promise<void>;
	startVerificationWithResult(
		phone: string,
		options?: SmsVerificationStartOptions,
	): Promise<SmsVerificationStartResult>;
	checkVerification(phone: string, code: string): Promise<boolean>;
	lookupPhone(phone: string): Promise<PhoneLookupResult | null>;
}
