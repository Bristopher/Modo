// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../../../BrandedTypes';
import type {IGatewayService} from '../../../infrastructure/IGatewayService';
import type {UserCacheService} from '../../../infrastructure/UserCacheService';
import type {RequestCache} from '../../../middleware/RequestCacheMiddleware';
import type {GuildMember} from '../../../models/GuildMember';
import {mapGuildMemberToResponse} from '../../GuildModel';

export class GuildMemberEventService {
	constructor(
		private readonly gatewayService: IGatewayService,
		private readonly userCacheService: UserCacheService,
	) {}

	async dispatchGuildMemberAdd({
		member,
		requestCache,
	}: {
		member: GuildMember;
		requestCache: RequestCache;
	}): Promise<void> {
		await this.gatewayService.dispatchGuild({
			guildId: member.guildId,
			event: 'GUILD_MEMBER_ADD',
			data: await mapGuildMemberToResponse(member, this.userCacheService, requestCache),
		});
	}

	async dispatchGuildMemberUpdate({
		guildId,
		member,
		requestCache,
	}: {
		guildId: GuildID;
		member: GuildMember;
		requestCache: RequestCache;
	}): Promise<void> {
		const memberResponse = await mapGuildMemberToResponse(member, this.userCacheService, requestCache);
		await this.gatewayService.dispatchGuild({
			guildId,
			event: 'GUILD_MEMBER_UPDATE',
			data: memberResponse,
		});
	}

	async dispatchGuildMemberRemove({guildId, userId}: {guildId: GuildID; userId: UserID}): Promise<void> {
		await this.gatewayService.dispatchGuild({
			guildId,
			event: 'GUILD_MEMBER_REMOVE',
			data: {user: {id: userId.toString()}},
		});
	}
}
