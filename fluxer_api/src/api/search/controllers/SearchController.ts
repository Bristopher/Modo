// SPDX-License-Identifier: AGPL-3.0-or-later

import {GlobalSearchMessagesRequest} from '@fluxer/schema/src/domains/message/MessageRequestSchemas';
import {MessageSearchResponse} from '@fluxer/schema/src/domains/message/MessageResponseSchemas';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function SearchController(app: HonoApp) {
	app.post(
		'/search/messages',
		RateLimitMiddleware(RateLimitConfigs.SEARCH_MESSAGES),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'search_messages',
			summary: 'Search messages',
			description: 'Searches for messages across guilds and channels accessible to the authenticated user.',
			responseSchema: MessageSearchResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Search',
		}),
		Validator('json', GlobalSearchMessagesRequest),
		async (ctx) => {
			const params = ctx.req.valid('json');
			const userId = ctx.get('user').id;
			const requestCache = ctx.get('requestCache');
			const result = await ctx.get('searchService').searchMessages({userId, requestCache, data: params});
			return ctx.json(result);
		},
	);
}
