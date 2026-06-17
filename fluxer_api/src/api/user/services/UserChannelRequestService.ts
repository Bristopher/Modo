// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ChannelResponse} from '@fluxer/schema/src/domains/channel/ChannelSchemas';
import type {CreatePrivateChannelRequest} from '@fluxer/schema/src/domains/user/UserRequestSchemas';
import type {ChannelID, UserID} from '../../BrandedTypes';
import {mapChannelToResponse} from '../../channel/ChannelMappers';
import type {UserCacheService} from '../../infrastructure/UserCacheService';
import type {RequestCache} from '../../middleware/RequestCacheMiddleware';
import type {UserChannelService} from './UserChannelService';

interface UserChannelListParams {
	userId: UserID;
	requestCache: RequestCache;
}

interface UserChannelCreateParams {
	userId: UserID;
	data: CreatePrivateChannelRequest;
	requestCache: RequestCache;
}

interface UserChannelPinParams {
	userId: UserID;
	channelId: ChannelID;
}

export class UserChannelRequestService {
	constructor(
		private readonly userChannelService: UserChannelService,
		private readonly userCacheService: UserCacheService,
	) {}

	async listPrivateChannels(params: UserChannelListParams): Promise<Array<ChannelResponse>> {
		const channels = await this.userChannelService.getPrivateChannels(params.userId);
		return Promise.all(
			channels.map((channel) =>
				mapChannelToResponse({
					channel,
					currentUserId: params.userId,
					userCacheService: this.userCacheService,
					requestCache: params.requestCache,
				}),
			),
		);
	}

	async createPrivateChannel(params: UserChannelCreateParams): Promise<ChannelResponse> {
		const channel = await this.userChannelService.createOrOpenDMChannel({
			userId: params.userId,
			data: params.data,
			userCacheService: this.userCacheService,
			requestCache: params.requestCache,
		});
		return mapChannelToResponse({
			channel,
			currentUserId: params.userId,
			userCacheService: this.userCacheService,
			requestCache: params.requestCache,
		});
	}

	async pinChannel(params: UserChannelPinParams): Promise<void> {
		await this.userChannelService.pinDmChannel({userId: params.userId, channelId: params.channelId});
	}

	async unpinChannel(params: UserChannelPinParams): Promise<void> {
		await this.userChannelService.unpinDmChannel({userId: params.userId, channelId: params.channelId});
	}
}
