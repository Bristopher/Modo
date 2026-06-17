// SPDX-License-Identifier: AGPL-3.0-or-later

import {UnfurlRequest, UnfurlResponse} from '@fluxer/schema/src/domains/unfurl/UnfurlSchemas';
import {DefaultUserOnly, LoginRequired} from '../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {OpenAPI} from '../middleware/ResponseTypeMiddleware';
import {getUnfurlerService} from '../middleware/ServiceSingletons';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoApp} from '../types/HonoEnv';
import {Validator} from '../Validator';

export function UnfurlController(app: HonoApp) {
	app.post(
		'/unfurl',
		RateLimitMiddleware(RateLimitConfigs.UNFURL_DEBUG),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'debug_unfurl',
			summary: 'Debug URL unfurl',
			responseSchema: UnfurlResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Debug'],
			description: 'Resolves a single URL through the unfurler without reading from the unfurl cache.',
		}),
		Validator('json', UnfurlRequest),
		async (ctx) => {
			const {url} = ctx.req.valid('json');
			const result = await getUnfurlerService().unfurlWithCachePolicy(url, 'block', {bypassCache: true});
			return ctx.json(result.embeds);
		},
	);
}
