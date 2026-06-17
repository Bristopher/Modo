// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ILogger} from '../ILogger';
import type {IAuditLogSearchService} from '../search/IAuditLogSearchService';
import type {IGuildMemberSearchService} from '../search/IGuildMemberSearchService';
import type {IGuildSearchService} from '../search/IGuildSearchService';
import type {IMessageSearchService} from '../search/IMessageSearchService';
import type {IReportSearchService} from '../search/IReportSearchService';
import type {ISearchProvider} from '../search/ISearchProvider';
import type {IUserSearchService} from '../search/IUserSearchService';
import {type MeilisearchClientConfig, MeilisearchHttpClient} from '../search/meilisearch/MeilisearchClient';
import {
	MeilisearchAuditLogSearchService,
	MeilisearchGuildMemberSearchService,
	MeilisearchGuildSearchService,
	MeilisearchMessageSearchService,
	MeilisearchReportSearchService,
	MeilisearchUserSearchService,
} from '../search/meilisearch/MeilisearchSearchServices';

interface MeilisearchSearchProviderOptions {
	config: MeilisearchClientConfig;
	logger: ILogger;
}

export class MeilisearchSearchProvider implements ISearchProvider {
	private readonly config: MeilisearchClientConfig;
	private readonly logger: ILogger;
	private messageService: MeilisearchMessageSearchService | null = null;
	private guildService: MeilisearchGuildSearchService | null = null;
	private userService: MeilisearchUserSearchService | null = null;
	private reportService: MeilisearchReportSearchService | null = null;
	private auditLogService: MeilisearchAuditLogSearchService | null = null;
	private guildMemberService: MeilisearchGuildMemberSearchService | null = null;

	constructor(options: MeilisearchSearchProviderOptions) {
		this.config = options.config;
		this.logger = options.logger;
	}

	async initialize(): Promise<void> {
		const client = new MeilisearchHttpClient(this.config);
		this.messageService = new MeilisearchMessageSearchService(client);
		const {GuildDiscoveryRepository} = await import('../guild/repositories/GuildDiscoveryRepository');
		this.guildService = new MeilisearchGuildSearchService(client, new GuildDiscoveryRepository());
		this.userService = new MeilisearchUserSearchService(client);
		this.reportService = new MeilisearchReportSearchService(client);
		this.auditLogService = new MeilisearchAuditLogSearchService(client);
		this.guildMemberService = new MeilisearchGuildMemberSearchService(client);
		await Promise.all([
			this.messageService.initialize(),
			this.guildService.initialize(),
			this.userService.initialize(),
			this.reportService.initialize(),
			this.auditLogService.initialize(),
			this.guildMemberService.initialize(),
		]);
		this.logger.info({host: this.config.host}, 'MeilisearchSearchProvider initialised');
	}

	async shutdown(): Promise<void> {
		const services = [
			this.messageService,
			this.guildService,
			this.userService,
			this.reportService,
			this.auditLogService,
			this.guildMemberService,
		];
		await Promise.all(services.filter((service) => service != null).map((service) => service.shutdown()));
		this.messageService = null;
		this.guildService = null;
		this.userService = null;
		this.reportService = null;
		this.auditLogService = null;
		this.guildMemberService = null;
	}

	getMessageSearchService(): IMessageSearchService | null {
		return this.messageService;
	}

	getGuildSearchService(): IGuildSearchService | null {
		return this.guildService;
	}

	getUserSearchService(): IUserSearchService | null {
		return this.userService;
	}

	getReportSearchService(): IReportSearchService | null {
		return this.reportService;
	}

	getAuditLogSearchService(): IAuditLogSearchService | null {
		return this.auditLogService;
	}

	getGuildMemberSearchService(): IGuildMemberSearchService | null {
		return this.guildMemberService;
	}
}
