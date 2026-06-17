// SPDX-License-Identifier: AGPL-3.0-or-later

export type SmsVerificationStartChannel = 'sms' | 'auto';

export interface SmsVerificationStartOptions {
	channel?: SmsVerificationStartChannel;
	deviceIp?: string;
	rateLimits?: Record<string, string>;
}

export interface SmsVerificationStartResult {
	channel: string;
}

export const SMS_VERIFICATION_START_SMS_RESULT: SmsVerificationStartResult = {
	channel: 'sms',
};
