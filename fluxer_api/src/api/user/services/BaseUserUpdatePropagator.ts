// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../../BrandedTypes';
import type {IGatewayService} from '../../infrastructure/IGatewayService';
import type {UserCacheService} from '../../infrastructure/UserCacheService';
import type {User} from '../../models/User';
import {invalidateUserCache, updateUserCache} from '../UserCacheHelpers';
import {mapUserToPrivateResponse} from '../UserMappers';

interface BaseUserUpdatePropagatorDeps {
	userCacheService: UserCacheService;
	gatewayService: IGatewayService;
}

export class BaseUserUpdatePropagator {
	constructor(protected readonly baseDeps: BaseUserUpdatePropagatorDeps) {}

	async dispatchUserUpdate(user: User): Promise<void> {
		await this.baseDeps.gatewayService.dispatchPresence({
			userId: user.id,
			event: 'USER_UPDATE',
			data: mapUserToPrivateResponse(user),
		});
	}

	async invalidateUserCache(userId: UserID): Promise<void> {
		await invalidateUserCache({
			userId,
			userCacheService: this.baseDeps.userCacheService,
		});
	}

	async updateUserCache(user: User): Promise<void> {
		await updateUserCache({
			user,
			userCacheService: this.baseDeps.userCacheService,
		});
	}
}
