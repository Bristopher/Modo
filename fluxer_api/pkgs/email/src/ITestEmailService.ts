// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IEmailService} from '@pkgs/email/src/IEmailService';

export interface SentEmailRecord {
	to: string;
	type: string;
	timestamp: Date;
	metadata: Record<string, string>;
}

export interface ITestEmailService extends IEmailService {
	listSentEmails(): Array<SentEmailRecord>;
	clearSentEmails(): void;
}
