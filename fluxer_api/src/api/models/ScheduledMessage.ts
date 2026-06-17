// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageReferenceTypeValue} from '@fluxer/constants/src/ChannelConstants';
import type {AllowedMentionsRequest} from '@fluxer/schema/src/domains/message/SharedMessageSchemas';
import type {ChannelID, MessageID, UserID} from '../BrandedTypes';
import type {AttachmentRequestData} from '../channel/AttachmentDTOs';
import type {MessageRequest} from '../channel/MessageTypes';
import type {ScheduledMessageRow} from '../database/types/UserTypes';
import {isJsonRecord, parseJsonWithGuard} from '../utils/JsonBoundaryUtils';

type ScheduledMessageStatus = 'pending' | 'invalid';

export interface ScheduledMessageReference {
	message_id: string;
	channel_id?: string;
	guild_id?: string;
	type?: MessageReferenceTypeValue;
}

export interface ScheduledAllowedMentions {
	parse?: AllowedMentionsRequest['parse'];
	users?: Array<string>;
	roles?: Array<string>;
	replied_user?: boolean;
}

export interface ScheduledMessagePayload {
	content?: string | null;
	embeds?: MessageRequest['embeds'];
	attachments?: Array<AttachmentRequestData>;
	message_reference?: ScheduledMessageReference;
	allowed_mentions?: ScheduledAllowedMentions;
	flags?: number;
	nonce?: string;
	favorite_meme_id?: string;
	sticker_ids?: Array<string>;
	tts?: boolean;
}

interface ScheduledMessageResponse {
	id: string;
	channel_id: string;
	scheduled_at: string;
	scheduled_local_at: string;
	timezone: string;
	status: ScheduledMessageStatus;
	status_reason: string | null;
	payload: ScheduledMessagePayload;
	created_at: string;
	invalidated_at: string | null;
}

export class ScheduledMessage {
	readonly userId: UserID;
	readonly id: MessageID;
	readonly channelId: ChannelID;
	readonly scheduledAt: Date;
	readonly scheduledLocalAt: string;
	readonly timezone: string;
	readonly payload: ScheduledMessagePayload;
	readonly status: ScheduledMessageStatus;
	readonly statusReason: string | null;
	readonly createdAt: Date;
	readonly invalidatedAt: Date | null;

	constructor(params: {
		userId: UserID;
		id: MessageID;
		channelId: ChannelID;
		scheduledAt: Date;
		scheduledLocalAt: string;
		timezone: string;
		payload: ScheduledMessagePayload;
		status?: ScheduledMessageStatus;
		statusReason?: string | null;
		createdAt?: Date;
		invalidatedAt?: Date | null;
	}) {
		this.userId = params.userId;
		this.id = params.id;
		this.channelId = params.channelId;
		this.scheduledAt = params.scheduledAt;
		this.scheduledLocalAt = params.scheduledLocalAt;
		this.timezone = params.timezone;
		this.payload = params.payload;
		this.status = params.status ?? 'pending';
		this.statusReason = params.statusReason ?? null;
		this.createdAt = params.createdAt ?? new Date();
		this.invalidatedAt = params.invalidatedAt ?? null;
	}

	static fromRow(row: ScheduledMessageRow): ScheduledMessage {
		return new ScheduledMessage({
			userId: row.user_id,
			id: row.scheduled_message_id,
			channelId: row.channel_id,
			scheduledAt: row.scheduled_at,
			scheduledLocalAt: row.scheduled_local_at,
			timezone: row.timezone,
			payload: parsePayload(row.payload),
			status: toStatus(row.status),
			statusReason: row.status_reason,
			createdAt: row.created_at,
			invalidatedAt: row.invalidated_at,
		});
	}

	toRow(): ScheduledMessageRow {
		return {
			user_id: this.userId,
			scheduled_message_id: this.id,
			channel_id: this.channelId,
			payload: JSON.stringify(this.payload),
			scheduled_at: this.scheduledAt,
			scheduled_local_at: this.scheduledLocalAt,
			timezone: this.timezone,
			status: this.status,
			status_reason: this.statusReason,
			created_at: this.createdAt,
			invalidated_at: this.invalidatedAt,
		};
	}

	toResponse(): ScheduledMessageResponse {
		return {
			id: this.id.toString(),
			channel_id: this.channelId.toString(),
			scheduled_at: this.scheduledAt.toISOString(),
			scheduled_local_at: this.scheduledLocalAt,
			timezone: this.timezone,
			status: this.status,
			status_reason: this.statusReason,
			payload: this.payload,
			created_at: this.createdAt.toISOString(),
			invalidated_at: this.invalidatedAt?.toISOString() ?? null,
		};
	}

	parseToMessageRequest(): MessageRequest {
		return hydrateMessageRequest(this.payload);
	}
}

function parsePayload(payload: string): ScheduledMessagePayload {
	return parseJsonWithGuard(payload, isScheduledMessagePayload) ?? {};
}

function isStringArray(value: unknown): value is Array<string> {
	return Array.isArray(value) && value.every((entry) => typeof entry === 'string');
}

function isScheduledMessageReference(value: unknown): value is ScheduledMessageReference {
	if (!isJsonRecord(value)) return false;
	return (
		typeof value.message_id === 'string' &&
		(value.channel_id === undefined || typeof value.channel_id === 'string') &&
		(value.guild_id === undefined || typeof value.guild_id === 'string') &&
		(value.type === undefined || typeof value.type === 'number')
	);
}

function isScheduledAllowedMentions(value: unknown): value is ScheduledAllowedMentions {
	if (!isJsonRecord(value)) return false;
	return (
		(value.parse === undefined || isStringArray(value.parse)) &&
		(value.users === undefined || isStringArray(value.users)) &&
		(value.roles === undefined || isStringArray(value.roles)) &&
		(value.replied_user === undefined || typeof value.replied_user === 'boolean')
	);
}

function isScheduledMessagePayload(value: unknown): value is ScheduledMessagePayload {
	if (!isJsonRecord(value)) return false;
	return (
		(value.content === undefined || value.content === null || typeof value.content === 'string') &&
		(value.embeds === undefined || Array.isArray(value.embeds)) &&
		(value.attachments === undefined || Array.isArray(value.attachments)) &&
		(value.message_reference === undefined || isScheduledMessageReference(value.message_reference)) &&
		(value.allowed_mentions === undefined || isScheduledAllowedMentions(value.allowed_mentions)) &&
		(value.flags === undefined || typeof value.flags === 'number') &&
		(value.nonce === undefined || typeof value.nonce === 'string') &&
		(value.favorite_meme_id === undefined || typeof value.favorite_meme_id === 'string') &&
		(value.sticker_ids === undefined || isStringArray(value.sticker_ids)) &&
		(value.tts === undefined || typeof value.tts === 'boolean')
	);
}

function toStatus(value: string): ScheduledMessageStatus {
	return value === 'invalid' ? 'invalid' : 'pending';
}

function hydrateMessageRequest(payload: ScheduledMessagePayload): MessageRequest {
	const message: MessageRequest = {
		content: payload.content,
		embeds: payload.embeds ?? [],
		attachments: payload.attachments,
		message_reference: payload.message_reference
			? {
					...payload.message_reference,
					message_id: BigInt(payload.message_reference.message_id),
					channel_id: payload.message_reference.channel_id ? BigInt(payload.message_reference.channel_id) : undefined,
					guild_id: payload.message_reference.guild_id ? BigInt(payload.message_reference.guild_id) : undefined,
				}
			: undefined,
		allowed_mentions: payload.allowed_mentions
			? {
					parse: payload.allowed_mentions.parse,
					users: payload.allowed_mentions.users?.map((value) => BigInt(value)),
					roles: payload.allowed_mentions.roles?.map((value) => BigInt(value)),
					replied_user: payload.allowed_mentions.replied_user,
				}
			: undefined,
		flags: payload.flags,
		nonce: payload.nonce,
		favorite_meme_id: payload.favorite_meme_id ? BigInt(payload.favorite_meme_id) : undefined,
		sticker_ids: payload.sticker_ids?.map((value) => BigInt(value)),
		tts: payload.tts,
	};
	return message;
}
