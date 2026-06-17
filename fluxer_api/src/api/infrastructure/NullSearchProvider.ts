// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IAuditLogSearchService} from '../search/IAuditLogSearchService';
import type {IGuildMemberSearchService} from '../search/IGuildMemberSearchService';
import type {IGuildSearchService} from '../search/IGuildSearchService';
import type {IMessageSearchService} from '../search/IMessageSearchService';
import type {IReportSearchService} from '../search/IReportSearchService';
import type {ISearchProvider} from '../search/ISearchProvider';
import type {IUserSearchService} from '../search/IUserSearchService';

export class NullSearchProvider implements ISearchProvider {
	async initialize(): Promise<void> {}

	async shutdown(): Promise<void> {}

	getMessageSearchService(): IMessageSearchService | null {
		return null;
	}

	getGuildSearchService(): IGuildSearchService | null {
		return null;
	}

	getUserSearchService(): IUserSearchService | null {
		return null;
	}

	getReportSearchService(): IReportSearchService | null {
		return null;
	}

	getAuditLogSearchService(): IAuditLogSearchService | null {
		return null;
	}

	getGuildMemberSearchService(): IGuildMemberSearchService | null {
		return null;
	}
}
