// SPDX-License-Identifier: AGPL-3.0-or-later

import {dispatchChannelEvent} from '@app/api/channel/services/ChannelGatewayDispatch';
import {GuildOperations} from '@fluxer/constants/src/GuildConstants';
import type {UserID} from '../../../BrandedTypes';
import type {Channel} from '../../../models/Channel';
import type {AuthenticatedChannel} from '../AuthenticatedChannel';
import {MessageInteractionBase} from './MessageInteractionBase';

export class MessageReadStateService extends MessageInteractionBase {
	async startTyping({authChannel, userId}: {authChannel: AuthenticatedChannel; userId: UserID}): Promise<void> {
		const {channel, guild} = authChannel;
		this.ensureTextChannel(channel);
		if (this.isOperationDisabled(guild, GuildOperations.TYPING_EVENTS)) {
			return;
		}
		await this.dispatchTypingStart({channel, userId});
	}

	private async dispatchTypingStart({channel, userId}: {channel: Channel; userId: UserID}): Promise<void> {
		await dispatchChannelEvent({
			gatewayService: this.gatewayService,
			channel,
			event: 'TYPING_START',
			data: {
				channel_id: channel.id.toString(),
				user_id: userId.toString(),
				timestamp: Math.floor(Date.now() / 1000),
			},
		});
	}
}
