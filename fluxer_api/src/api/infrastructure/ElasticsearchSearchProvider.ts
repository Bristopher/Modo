// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ElasticsearchDistributedLock} from '@pkgs/elasticsearch_search/src/adapters/ElasticsearchIndexAdapter';
import {
	createElasticsearchClient,
	type ElasticsearchClientConfig,
} from '@pkgs/elasticsearch_search/src/ElasticsearchClient';
import type {ILogger} from '../ILogger';
import {ElasticsearchAuditLogSearchService} from '../search/elasticsearch/ElasticsearchAuditLogSearchService';
import {ElasticsearchGuildMemberSearchService} from '../search/elasticsearch/ElasticsearchGuildMemberSearchService';
import {ElasticsearchGuildSearchService} from '../search/elasticsearch/ElasticsearchGuildSearchService';
import {ElasticsearchMessageSearchService} from '../search/elasticsearch/ElasticsearchMessageSearchService';
import {ElasticsearchReportSearchService} from '../search/elasticsearch/ElasticsearchReportSearchService';
import {ElasticsearchUserSearchService} from '../search/elasticsearch/ElasticsearchUserSearchService';
import type {IAuditLogSearchService} from '../search/IAuditLogSearchService';
import type {IGuildMemberSearchService} from '../search/IGuildMemberSearchService';
import type {IGuildSearchService} from '../search/IGuildSearchService';
import type {IMessageSearchService} from '../search/IMessageSearchService';
import type {IReportSearchService} from '../search/IReportSearchService';
import type {ISearchProvider} from '../search/ISearchProvider';
import type {IUserSearchService} from '../search/IUserSearchService';

interface ElasticsearchSearchProviderOptions {
	config: ElasticsearchClientConfig;
	logger: ILogger;
	lock?: ElasticsearchDistributedLock;
}

export class ElasticsearchSearchProvider implements ISearchProvider {
	private readonly logger: ILogger;
	private readonly config: ElasticsearchClientConfig;
	private readonly lock: ElasticsearchDistributedLock | undefined;
	private messageService: ElasticsearchMessageSearchService | null = null;
	private guildService: ElasticsearchGuildSearchService | null = null;
	private userService: ElasticsearchUserSearchService | null = null;
	private reportService: ElasticsearchReportSearchService | null = null;
	private auditLogService: ElasticsearchAuditLogSearchService | null = null;
	private guildMemberService: ElasticsearchGuildMemberSearchService | null = null;

	constructor(options: ElasticsearchSearchProviderOptions) {
		this.logger = options.logger;
		this.config = options.config;
		this.lock = options.lock;
	}

	async initialize(): Promise<void> {
		const client = createElasticsearchClient(this.config);
		const lock = this.lock;
		this.messageService = new ElasticsearchMessageSearchService({client, lock});
		const {GuildDiscoveryRepository} = await import('../guild/repositories/GuildDiscoveryRepository');
		this.guildService = new ElasticsearchGuildSearchService({
			client,
			lock,
			discoveryRepository: new GuildDiscoveryRepository(),
		});
		this.userService = new ElasticsearchUserSearchService({client, lock});
		this.reportService = new ElasticsearchReportSearchService({client, lock});
		this.auditLogService = new ElasticsearchAuditLogSearchService({client, lock});
		this.guildMemberService = new ElasticsearchGuildMemberSearchService({client, lock});
		await Promise.all([
			this.messageService.initialize(),
			this.guildService.initialize(),
			this.userService.initialize(),
			this.reportService.initialize(),
			this.auditLogService.initialize(),
			this.guildMemberService.initialize(),
		]);
		this.logger.info({node: this.config.node}, 'ElasticsearchSearchProvider initialised');
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
		await Promise.all(services.filter((s) => s != null).map((s) => s.shutdown()));
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
