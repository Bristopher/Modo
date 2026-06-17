// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IpInfoService} from '@pkgs/geoip/src/IpInfoService';
import type {IVirusScanService} from '@pkgs/virus_scan/src/IVirusScanService';
import type {ApiContext} from '../ApiContext';
import type {IChannelRepository} from '../channel/IChannelRepository';
import type {AttachmentUploadTraceRepository} from '../channel/repositories/message/AttachmentUploadTraceRepository';
import {ChannelService} from '../channel/services/ChannelService';
import type {IFavoriteMemeRepository} from '../favorite_meme/IFavoriteMemeRepository';
import type {GuildAuditLogService} from '../guild/GuildAuditLogService';
import type {IGuildRepositoryAggregate} from '../guild/repositories/IGuildRepositoryAggregate';
import type {ExpressionAssetPurger} from '../guild/services/content/ExpressionAssetPurger';
import {GuildService} from '../guild/services/GuildService';
import type {AvatarService} from '../infrastructure/AvatarService';
import type {IPurgeQueue} from '../infrastructure/BunnyPurgeQueue';
import type {EmbedService} from '../infrastructure/EmbedService';
import type {EntityAssetService} from '../infrastructure/EntityAssetService';
import type {IAssetDeletionQueue} from '../infrastructure/IAssetDeletionQueue';
import type {ILiveKitService} from '../infrastructure/ILiveKitService';
import type {IStorageService} from '../infrastructure/IStorageService';
import type {IVoiceRoomStore} from '../infrastructure/IVoiceRoomStore';
import type {UserCacheService} from '../infrastructure/UserCacheService';
import type {InviteRepository} from '../invite/InviteRepository';
import {InviteService} from '../invite/InviteService';
import type {LimitConfigService} from '../limits/LimitConfigService';
import type {PackRepository} from '../pack/PackRepository';
import {PackService} from '../pack/PackService';
import type {ReadStateService} from '../read_state/ReadStateService';
import type {IUserRepository} from '../user/IUserRepository';
import type {VoiceAvailabilityService} from '../voice/VoiceAvailabilityService';
import type {IWebhookRepository} from '../webhook/IWebhookRepository';

interface GuildStackServiceFactoryDependencies {
	apiContext: ApiContext;
	packRepository: PackRepository;
	channelRepository: IChannelRepository;
	userRepository: IUserRepository;
	guildRepository: IGuildRepositoryAggregate;
	inviteRepository: InviteRepository;
	webhookRepository: IWebhookRepository;
	favoriteMemeRepository: IFavoriteMemeRepository;
	avatarService: AvatarService;
	entityAssetService: EntityAssetService;
	assetDeletionQueue: IAssetDeletionQueue;
	expressionAssetPurger: ExpressionAssetPurger;
	userCacheService: UserCacheService;
	limitConfigService: LimitConfigService;
	embedService: EmbedService;
	readStateService: ReadStateService;
	storageService: IStorageService;
	attachmentUploadTraceRepository: AttachmentUploadTraceRepository;
	virusScanService: IVirusScanService;
	purgeQueue: IPurgeQueue;
	guildAuditLogService: GuildAuditLogService;
	voiceRoomStore: IVoiceRoomStore;
	liveKitService: ILiveKitService;
	voiceAvailabilityService: VoiceAvailabilityService | null;
	ipInfoService: IpInfoService;
}

interface GuildStackServices {
	packService: PackService;
	channelService: ChannelService;
	guildService: GuildService;
	inviteService: InviteService;
}

export function createGuildStackServices(dependencies: GuildStackServiceFactoryDependencies): GuildStackServices {
	const packService = new PackService(
		dependencies.apiContext,
		dependencies.packRepository,
		dependencies.guildRepository,
		dependencies.avatarService,
		dependencies.expressionAssetPurger,
		dependencies.userCacheService,
		dependencies.limitConfigService,
	);
	const channelService = new ChannelService(
		dependencies.apiContext,
		dependencies.channelRepository,
		dependencies.userRepository,
		dependencies.guildRepository,
		packService,
		dependencies.userCacheService,
		dependencies.embedService,
		dependencies.readStateService,
		dependencies.storageService,
		dependencies.attachmentUploadTraceRepository,
		dependencies.avatarService,
		dependencies.virusScanService,
		dependencies.purgeQueue,
		dependencies.favoriteMemeRepository,
		dependencies.guildAuditLogService,
		dependencies.voiceRoomStore,
		dependencies.liveKitService,
		dependencies.inviteRepository,
		dependencies.webhookRepository,
		dependencies.limitConfigService,
		dependencies.voiceAvailabilityService,
	);
	const guildService = new GuildService(
		dependencies.apiContext,
		dependencies.guildRepository,
		dependencies.channelRepository,
		dependencies.inviteRepository,
		channelService,
		dependencies.userCacheService,
		dependencies.entityAssetService,
		dependencies.avatarService,
		dependencies.assetDeletionQueue,
		dependencies.webhookRepository,
		dependencies.guildAuditLogService,
		dependencies.limitConfigService,
		dependencies.ipInfoService,
	);
	const inviteService = new InviteService(
		dependencies.apiContext,
		dependencies.inviteRepository,
		guildService,
		channelService,
		dependencies.guildAuditLogService,
		dependencies.packRepository,
		packService,
		dependencies.limitConfigService,
	);
	return {
		packService,
		channelService,
		guildService,
		inviteService,
	};
}
