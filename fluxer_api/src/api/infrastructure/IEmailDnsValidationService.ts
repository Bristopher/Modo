// SPDX-License-Identifier: AGPL-3.0-or-later

export interface IEmailDnsValidationService {
	hasValidDnsRecords(email: string): Promise<boolean>;
}
