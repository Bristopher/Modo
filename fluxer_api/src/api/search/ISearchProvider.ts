// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IAuditLogSearchService} from './IAuditLogSearchService';
import type {IGuildMemberSearchService} from './IGuildMemberSearchService';
import type {IGuildSearchService} from './IGuildSearchService';
import type {IMessageSearchService} from './IMessageSearchService';
import type {IReportSearchService} from './IReportSearchService';
import type {IUserSearchService} from './IUserSearchService';

export interface ISearchProvider {
	initialize(): Promise<void>;
	shutdown(): Promise<void>;
	getMessageSearchService(): IMessageSearchService | null;
	getGuildSearchService(): IGuildSearchService | null;
	getUserSearchService(): IUserSearchService | null;
	getReportSearchService(): IReportSearchService | null;
	getAuditLogSearchService(): IAuditLogSearchService | null;
	getGuildMemberSearchService(): IGuildMemberSearchService | null;
}
