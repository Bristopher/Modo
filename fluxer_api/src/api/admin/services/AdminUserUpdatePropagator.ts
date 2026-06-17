// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../../BrandedTypes';
import {mapGuildMemberToResponse} from '../../guild/GuildModel';
import type {IGuildRepositoryAggregate} from '../../guild/repositories/IGuildRepositoryAggregate';
import type {IGatewayService} from '../../infrastructure/IGatewayService';
import type {UserCacheService} from '../../infrastructure/UserCacheService';
import {createRequestCache} from '../../middleware/RequestCacheMiddleware';
import type {User} from '../../models/User';
import type {IUserRepository} from '../../user/IUserRepository';
import {BaseUserUpdatePropagator} from '../../user/services/BaseUserUpdatePropagator';
import {hasPartialUserFieldsChanged} from '../../user/UserMappers';

interface AdminUserUpdatePropagatorDeps {
	userCacheService: UserCacheService;
	userRepository: IUserRepository;
	guildRepository: IGuildRepositoryAggregate;
	gatewayService: IGatewayService;
}

export class AdminUserUpdatePropagator extends BaseUserUpdatePropagator {
	constructor(private readonly deps: AdminUserUpdatePropagatorDeps) {
		super({
			userCacheService: deps.userCacheService,
			gatewayService: deps.gatewayService,
		});
	}

	async propagateUserUpdate({
		userId,
		oldUser,
		updatedUser,
	}: {
		userId: UserID;
		oldUser: User;
		updatedUser: User;
	}): Promise<void> {
		await this.dispatchUserUpdate(updatedUser);
		if (hasPartialUserFieldsChanged(oldUser, updatedUser)) {
			await this.updateUserCache(updatedUser);
			await this.propagateToGuilds(userId);
		}
	}

	private async propagateToGuilds(userId: UserID): Promise<void> {
		const {userRepository, guildRepository, gatewayService, userCacheService} = this.deps;
		const guildIds = await userRepository.getUserGuildIds(userId);
		if (guildIds.length === 0) {
			return;
		}
		const requestCache = createRequestCache();
		for (const guildId of guildIds) {
			const member = await guildRepository.getMember(guildId, userId);
			if (!member) {
				continue;
			}
			const memberResponse = await mapGuildMemberToResponse(member, userCacheService, requestCache);
			await gatewayService.dispatchGuild({
				guildId,
				event: 'GUILD_MEMBER_UPDATE',
				data: memberResponse,
			});
		}
		requestCache.clear();
	}
}
