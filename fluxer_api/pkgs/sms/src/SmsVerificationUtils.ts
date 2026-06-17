// SPDX-License-Identifier: AGPL-3.0-or-later

import {SMS_MASK_VISIBLE_PREFIX_LENGTH} from '@fluxer/constants/src/SmsVerificationConstants';

export function maskPhoneNumber(phone: string): string {
	if (phone.length <= SMS_MASK_VISIBLE_PREFIX_LENGTH) {
		return `${phone}***`;
	}
	return `${phone.slice(0, SMS_MASK_VISIBLE_PREFIX_LENGTH)}***`;
}
