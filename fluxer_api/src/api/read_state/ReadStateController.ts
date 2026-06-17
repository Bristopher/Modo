// SPDX-License-Identifier: AGPL-3.0-or-later

import {
	ReadStateAckBulkRequest,
	ReadStateAckRequest,
	ReadStateAckResponse,
} from '@fluxer/schema/src/domains/channel/ChannelRequestSchemas';
import type {Hono} from 'hono';
import {DefaultUserOnly, LoginRequired} from '../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {OpenAPI} from '../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoEnv} from '../types/HonoEnv';
import {Validator} from '../Validator';

export function ReadStateController(app: Hono<HonoEnv>): void {
	app.post(
		'/read-states/ack',
		RateLimitMiddleware(RateLimitConfigs.READ_STATE_ACK_BULK),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'ack_read_states',
			summary: 'Acknowledge read states',
			description:
				'Applies one or more read-state acknowledgements and returns the authoritative read states after the write.',
			responseSchema: ReadStateAckResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Read States'],
		}),
		Validator('json', ReadStateAckRequest),
		async (ctx) => {
			return ctx.json(
				await ctx.get('readStateRequestService').ackReadStates({
					userId: ctx.get('user').id,
					data: ctx.req.valid('json'),
				}),
			);
		},
	);
	app.post(
		'/read-states/ack-bulk',
		RateLimitMiddleware(RateLimitConfigs.READ_STATE_ACK_BULK),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'ack_bulk_messages',
			summary: 'Mark channels as read',
			description: 'Marks multiple channels as read for the authenticated user in bulk.',
			responseSchema: null,
			statusCode: 204,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Read States'],
		}),
		Validator('json', ReadStateAckBulkRequest),
		async (ctx) => {
			await ctx.get('readStateRequestService').bulkAckMessages({
				userId: ctx.get('user').id,
				data: ctx.req.valid('json'),
			});
			return ctx.body(null, 204);
		},
	);
}
