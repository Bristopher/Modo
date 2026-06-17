// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ApiContext} from '../../ApiContext';
import type {IChannelRepository} from '../../channel/IChannelRepository';
import type {ChannelService} from '../../channel/services/ChannelService';
import type {IConnectionRepository} from '../../connection/IConnectionRepository';
import type {IGuildRepositoryAggregate} from '../../guild/repositories/IGuildRepositoryAggregate';
import type {GuildService} from '../../guild/services/GuildService';
import type {IDiscriminatorService} from '../../infrastructure/DiscriminatorService';
import type {EntityAssetService} from '../../infrastructure/EntityAssetService';
import type {KVAccountDeletionQueueService} from '../../infrastructure/KVAccountDeletionQueueService';
import type {KVBulkMessageDeletionQueueService} from '../../infrastructure/KVBulkMessageDeletionQueueService';
import type {UserCacheService} from '../../infrastructure/UserCacheService';
import type {LimitConfigService} from '../../limits/LimitConfigService';
import type {UserPermissionUtils} from '../../utils/UserPermissionUtils';
import {UserAccountService} from './UserAccountService';
import {UserChannelService} from './UserChannelService';
import type {UserContactChangeLogService} from './UserContactChangeLogService';
import {UserContentService} from './UserContentService';
import {UserRelationshipService} from './UserRelationshipService';

export class UserService {
	public readonly accountService: UserAccountService;
	public readonly relationshipService: UserRelationshipService;
	public readonly channelService: UserChannelService;
	public readonly contentService: UserContentService;

	constructor(
		apiContext: ApiContext,
		userCacheService: UserCacheService,
		channelService: ChannelService,
		channelRepository: IChannelRepository,
		guildService: GuildService,
		entityAssetService: EntityAssetService,
		discriminatorService: IDiscriminatorService,
		guildRepository: IGuildRepositoryAggregate,
		userPermissionUtils: UserPermissionUtils,
		kvDeletionQueue: KVAccountDeletionQueueService,
		bulkMessageDeletionQueue: KVBulkMessageDeletionQueueService,
		contactChangeLogService: UserContactChangeLogService,
		connectionRepository: IConnectionRepository,
		limitConfigService: LimitConfigService,
	) {
		this.accountService = new UserAccountService(
			apiContext,
			userCacheService,
			guildService,
			entityAssetService,
			guildRepository,
			discriminatorService,
			kvDeletionQueue,
			contactChangeLogService,
			connectionRepository,
			limitConfigService,
		);
		this.relationshipService = new UserRelationshipService(apiContext, userPermissionUtils, limitConfigService);
		this.channelService = new UserChannelService(
			apiContext,
			channelService,
			channelRepository,
			userPermissionUtils,
			limitConfigService,
		);
		this.contentService = new UserContentService(
			apiContext,
			userCacheService,
			channelService,
			channelRepository,
			bulkMessageDeletionQueue,
			limitConfigService,
		);
	}
}
