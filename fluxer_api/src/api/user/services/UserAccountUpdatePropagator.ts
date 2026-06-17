// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../../BrandedTypes';
import type {IGatewayService} from '../../infrastructure/IGatewayService';
import type {IMediaService} from '../../infrastructure/IMediaService';
import type {UserCacheService} from '../../infrastructure/UserCacheService';
import type {UserGuildSettings} from '../../models/UserGuildSettings';
import type {UserSettings} from '../../models/UserSettings';
import type {IUserAccountRepository} from '../repositories/IUserAccountRepository';
import {mapUserGuildSettingsToResponse, mapUserSettingsToResponse} from '../UserMappers';
import {BaseUserUpdatePropagator} from './BaseUserUpdatePropagator';

interface UserAccountUpdatePropagatorDeps {
	userCacheService: UserCacheService;
	gatewayService: IGatewayService;
	mediaService: IMediaService;
	userRepository: IUserAccountRepository;
}

export class UserAccountUpdatePropagator extends BaseUserUpdatePropagator {
	constructor(private readonly deps: UserAccountUpdatePropagatorDeps) {
		super({
			userCacheService: deps.userCacheService,
			gatewayService: deps.gatewayService,
		});
	}

	async dispatchUserSettingsUpdate({userId, settings}: {userId: UserID; settings: UserSettings}): Promise<void> {
		await this.deps.gatewayService.dispatchPresence({
			userId,
			event: 'USER_SETTINGS_UPDATE',
			data: mapUserSettingsToResponse({settings}),
		});
	}

	async dispatchUserGuildSettingsUpdate({
		userId,
		settings,
	}: {
		userId: UserID;
		settings: UserGuildSettings;
	}): Promise<void> {
		const payload = mapUserGuildSettingsToResponse(settings);
		await this.deps.gatewayService.dispatchPresence({
			userId,
			event: 'USER_GUILD_SETTINGS_UPDATE',
			data: payload,
		});
		if (payload.guild_id !== null) {
			await this.deps.gatewayService.syncPushUserGuildSettings({
				userId,
				guildId: settings.guildId,
				settings: payload,
			});
		}
	}

	async dispatchUserNoteUpdate(params: {userId: UserID; targetId: UserID; note: string}): Promise<void> {
		const {userId, targetId, note} = params;
		await this.deps.gatewayService.dispatchPresence({
			userId,
			event: 'USER_NOTE_UPDATE',
			data: {id: targetId.toString(), note},
		});
	}
}
