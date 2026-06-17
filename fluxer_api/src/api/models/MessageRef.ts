// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageReferenceTypeValue} from '@fluxer/constants/src/ChannelConstants';
import type {ChannelID, GuildID, MessageID} from '../BrandedTypes';
import type {MessageReference} from '../database/types/MessageTypes';

export class MessageRef {
	readonly channelId: ChannelID;
	readonly messageId: MessageID;
	readonly guildId: GuildID | null;
	readonly type: MessageReferenceTypeValue;

	constructor(ref: MessageReference) {
		this.channelId = ref.channel_id;
		this.messageId = ref.message_id;
		this.guildId = ref.guild_id ?? null;
		this.type = ref.type as MessageReferenceTypeValue;
	}

	toMessageReference(): MessageReference {
		return {
			channel_id: this.channelId,
			message_id: this.messageId,
			guild_id: this.guildId,
			type: this.type,
		};
	}
}
