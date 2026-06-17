// SPDX-License-Identifier: AGPL-3.0-or-later

export type PhoneLineType =
	| 'mobile'
	| 'landline'
	| 'fixedVoip'
	| 'nonFixedVoip'
	| 'personal'
	| 'tollFree'
	| 'premium'
	| 'sharedCost'
	| 'uan'
	| 'voicemail'
	| 'pager'
	| 'unknown';

export interface PhoneLookupResult {
	valid: boolean;
	lineType: PhoneLineType | null;
	countryCode: string | null;
	carrierName: string | null;
	smsPumpingRiskScore: number | null;
}

export const ACCEPTED_PHONE_LINE_TYPES: ReadonlySet<PhoneLineType> = new Set<PhoneLineType>(['mobile', 'personal']);
export const VOIP_PHONE_LINE_TYPES: ReadonlySet<PhoneLineType> = new Set<PhoneLineType>(['fixedVoip', 'nonFixedVoip']);
export const HARD_REJECT_PHONE_LINE_TYPES: ReadonlySet<PhoneLineType> = new Set<PhoneLineType>([
	'landline',
	'tollFree',
	'premium',
	'sharedCost',
	'uan',
	'voicemail',
	'pager',
]);
const SMS_PUMPING_RISK_THRESHOLDS: Readonly<Record<string, number>> = {
	US: 100,
	CA: 100,
	GB: 70,
	DE: 70,
	FR: 70,
	IT: 70,
	ES: 70,
	NL: 70,
	SE: 70,
	NO: 70,
	DK: 70,
	FI: 70,
	AU: 70,
	NZ: 70,
	JP: 70,
	KR: 70,
	CH: 70,
	AT: 70,
	BE: 70,
	IE: 70,
	PT: 70,
};
const DEFAULT_SMS_PUMPING_RISK_THRESHOLD = 35;

export function getSmsPumpingRiskThreshold(countryCode: string | null): number {
	if (countryCode === null) return DEFAULT_SMS_PUMPING_RISK_THRESHOLD;
	return SMS_PUMPING_RISK_THRESHOLDS[countryCode] ?? DEFAULT_SMS_PUMPING_RISK_THRESHOLD;
}
