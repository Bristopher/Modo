// SPDX-License-Identifier: AGPL-3.0-or-later

import {ChannelIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {
	ScheduledMessageRequestSchema,
	ScheduledMessageResponseSchema,
} from '@fluxer/schema/src/domains/message/ScheduledMessageSchemas';
import {createChannelID} from '../../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';
import {parseScheduledMessageInput} from './ScheduledMessageParsing';

export function ScheduledMessageController(app: HonoApp) {
	app.post(
		'/channels/:channel_id/messages/schedule',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_MESSAGE_CREATE),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		OpenAPI({
			operationId: 'schedule_message',
			summary: 'Schedule a message to send later',
			description:
				'Schedules a message to be sent at a specified time. Only available for regular user accounts. Requires permission to send messages in the target channel. Message is sent automatically at the scheduled time. Returns the scheduled message object with delivery time.',
			responseSchema: ScheduledMessageResponseSchema,
			requestSchema: ScheduledMessageRequestSchema,
			requestFormSchema: ScheduledMessageRequestSchema,
			statusCode: 201,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const user = ctx.get('user');
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const scheduledMessageService = ctx.get('scheduledMessageService');
			const {message, scheduledLocalAt, timezone} = await parseScheduledMessageInput({
				ctx,
				user,
				channelId,
			});
			const scheduledMessage = await scheduledMessageService.createScheduledMessage({
				user,
				channelId,
				data: message,
				scheduledLocalAt,
				timezone,
			});
			return ctx.json(scheduledMessage.toResponse(), 201);
		},
	);
}
