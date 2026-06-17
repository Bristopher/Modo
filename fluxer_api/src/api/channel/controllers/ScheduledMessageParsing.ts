// SPDX-License-Identifier: AGPL-3.0-or-later

import {ValidationErrorCodes} from '@fluxer/constants/src/ValidationErrorCodes';
import {InputValidationError} from '@fluxer/errors/src/domains/core/InputValidationError';
import {MessageRequestSchema} from '@fluxer/schema/src/domains/message/MessageRequestSchemas';
import {ScheduledMessageRequestSchema} from '@fluxer/schema/src/domains/message/ScheduledMessageSchemas';
import type {Context} from 'hono';
import {ms} from 'itty-time';
import type {ChannelID} from '../../BrandedTypes';
import type {User} from '../../models/User';
import type {HonoEnv} from '../../types/HonoEnv';
import {parseJsonPreservingLargeIntegers} from '../../utils/LosslessJsonParser';
import type {MessageRequest} from '../MessageTypes';
import {normalizeMessageRequestPayload} from '../services/message/MessageRequestCompatibility';
import {parseMultipartMessageData} from '../services/message/MessageRequestParser';

type ScheduledMessageSchemaType = ScheduledMessageRequestSchema;

function extractScheduleFields(data: ScheduledMessageSchemaType): {
	scheduled_local_at: string;
	timezone: string;
	message: MessageRequest;
} {
	const {scheduled_local_at, timezone, ...messageData} = data;
	return {
		scheduled_local_at,
		timezone,
		message: messageData as MessageRequest,
	};
}

export async function parseScheduledMessageInput({
	ctx,
	user,
	channelId,
}: {
	ctx: Context<HonoEnv>;
	user: User;
	channelId: ChannelID;
}): Promise<{
	message: MessageRequest;
	scheduledLocalAt: string;
	timezone: string;
}> {
	const contentType = ctx.req.header('content-type') ?? '';
	const isMultipart = contentType.includes('multipart/form-data');
	if (isMultipart) {
		let parsedPayload: unknown = null;
		const message = (await parseMultipartMessageData(ctx, user, channelId, MessageRequestSchema, {
			uploadExpiresAt: new Date(Date.now() + ms('32 days')),
			onPayloadParsed(payload) {
				parsedPayload = payload;
			},
		})) as MessageRequest;
		if (!parsedPayload) {
			throw InputValidationError.fromCode('scheduled_message', ValidationErrorCodes.FAILED_TO_PARSE_MULTIPART_PAYLOAD);
		}
		const validation = ScheduledMessageRequestSchema.safeParse(normalizeMessageRequestPayload(parsedPayload));
		if (!validation.success) {
			throw InputValidationError.fromCode('scheduled_message', ValidationErrorCodes.INVALID_SCHEDULED_MESSAGE_PAYLOAD);
		}
		const {scheduled_local_at, timezone} = extractScheduleFields(validation.data);
		return {message, scheduledLocalAt: scheduled_local_at, timezone};
	}
	let body: unknown;
	try {
		const raw = await ctx.req.text();
		body = raw.trim().length === 0 ? {} : parseJsonPreservingLargeIntegers(raw);
	} catch {
		throw InputValidationError.fromCode('scheduled_message', ValidationErrorCodes.INVALID_SCHEDULED_MESSAGE_PAYLOAD);
	}
	const validation = ScheduledMessageRequestSchema.safeParse(normalizeMessageRequestPayload(body));
	if (!validation.success) {
		throw InputValidationError.fromCode('scheduled_message', ValidationErrorCodes.INVALID_SCHEDULED_MESSAGE_PAYLOAD);
	}
	const {scheduled_local_at, timezone, message} = extractScheduleFields(validation.data);
	return {message, scheduledLocalAt: scheduled_local_at, timezone};
}
