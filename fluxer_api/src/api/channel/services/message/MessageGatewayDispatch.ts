// SPDX-License-Identifier: AGPL-3.0-or-later

import {ChannelTypes} from '@fluxer/constants/src/ChannelConstants';
import type {MessageResponse} from '@fluxer/schema/src/domains/message/MessageResponseSchemas';
import type {GuildID, UserID} from '../../../BrandedTypes';
import type {IGatewayService} from '../../../infrastructure/IGatewayService';
import type {Channel} from '../../../models/Channel';
import type {Message} from '../../../models/Message';
import {dispatchChannelEvent} from '../ChannelGatewayDispatch';
import {createMessageResponseDataService} from './MessageResponseDataService';

interface BuildBroadcastMessageDataParams {
	channel: Channel;
	message: Message;
	currentUserId?: UserID;
	nonce?: string;
	tts?: boolean;
	sourceGuildId?: GuildID | null;
}

interface BuildBroadcastMessageCreateDataParams extends BuildBroadcastMessageDataParams {
	mentionHere?: boolean;
}

type MessageCreateBroadcastData = MessageResponse & {
	channel_type: Channel['type'];
	nicks?: Record<string, string>;
	mention_here?: true;
};

export async function buildBroadcastMessageData({
	channel,
	message,
	currentUserId,
	nonce,
	tts,
	sourceGuildId,
}: BuildBroadcastMessageDataParams): Promise<MessageResponse> {
	return createMessageResponseDataService().buildBroadcastMessage({
		channel,
		message,
		userId: currentUserId,
		nonce,
		tts,
		sourceGuildId,
	});
}

async function buildBroadcastMessageCreateData(
	params: BuildBroadcastMessageCreateDataParams,
): Promise<MessageCreateBroadcastData> {
	const messageResponse = await buildBroadcastMessageData(params);
	const groupDmNicks = serializeGroupDmNicks(params.channel);
	return {
		...messageResponse,
		channel_type: params.channel.type,
		...(groupDmNicks ? {nicks: groupDmNicks} : {}),
		...(params.mentionHere ? {mention_here: true} : {}),
	};
}

function serializeGroupDmNicks(channel: Channel): Record<string, string> | undefined {
	if (channel.type !== ChannelTypes.GROUP_DM || !channel.nicknames || channel.nicknames.size === 0) {
		return undefined;
	}
	const nicks: Record<string, string> = {};
	for (const [userId, nickname] of channel.nicknames) {
		nicks[String(userId)] = nickname;
	}
	return Object.keys(nicks).length > 0 ? nicks : undefined;
}

export async function dispatchMessageCreateBroadcast({
	gatewayService,
	...params
}: BuildBroadcastMessageCreateDataParams & {
	gatewayService: IGatewayService;
}): Promise<void> {
	await dispatchChannelEvent({
		gatewayService,
		channel: params.channel,
		event: 'MESSAGE_CREATE',
		data: await buildBroadcastMessageCreateData(params),
	});
}

export async function dispatchMessageCreateToUser({
	gatewayService,
	userId,
	...params
}: BuildBroadcastMessageCreateDataParams & {
	gatewayService: IGatewayService;
	userId: UserID;
}): Promise<void> {
	await gatewayService.dispatchPresence({
		userId,
		event: 'MESSAGE_CREATE',
		data: await buildBroadcastMessageCreateData(params),
	});
}

export async function dispatchMessageUpdateBroadcast({
	gatewayService,
	...params
}: BuildBroadcastMessageDataParams & {
	gatewayService: IGatewayService;
}): Promise<void> {
	await dispatchChannelEvent({
		gatewayService,
		channel: params.channel,
		event: 'MESSAGE_UPDATE',
		data: await buildBroadcastMessageData(params),
	});
}
