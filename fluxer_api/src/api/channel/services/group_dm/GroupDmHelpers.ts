// SPDX-License-Identifier: AGPL-3.0-or-later

import {dispatchChannelEvent} from '@app/api/channel/services/ChannelGatewayDispatch';
import type {IGatewayService} from '../../../infrastructure/IGatewayService';
import type {UserCacheService} from '../../../infrastructure/UserCacheService';
import type {RequestCache} from '../../../middleware/RequestCacheMiddleware';
import type {Channel} from '../../../models/Channel';
import {mapChannelToResponse} from '../../ChannelMappers';

export async function dispatchChannelDelete({
	channel,
	requestCache,
	userCacheService,
	gatewayService,
}: {
	channel: Channel;
	requestCache: RequestCache;
	userCacheService: UserCacheService;
	gatewayService: IGatewayService;
}): Promise<void> {
	const channelResponse = await mapChannelToResponse({
		channel,
		currentUserId: null,
		userCacheService,
		requestCache,
	});
	await dispatchChannelEvent({
		channel,
		event: 'CHANNEL_DELETE',
		data: channelResponse,
		gatewayService,
	});
}
