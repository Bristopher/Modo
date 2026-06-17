// SPDX-License-Identifier: AGPL-3.0-or-later

export type HistoricalOutcomeCode = 'challenged' | 'spammer' | 'disabled' | 'disabled_suspicious';

export interface LatestRiskContextRecord {
	userId: string;
	ip: string;
	subnet: string | null;
	emailDomain: string | null;
	asn: number | null;
	updatedAt: Date;
}

export interface HistoricalOutcomeRecord {
	userId: string;
	createdAt: Date;
	outcomeCode: HistoricalOutcomeCode;
	source: string;
}
