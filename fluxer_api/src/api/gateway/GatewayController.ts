// SPDX-License-Identifier: AGPL-3.0-or-later

import {GatewayBotResponse} from '@fluxer/schema/src/domains/gateway/GatewaySchemas';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {OpenAPI} from '../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoApp} from '../types/HonoEnv';

export function GatewayController(app: HonoApp) {
	app.get(
		'/gateway/bot',
		RateLimitMiddleware(RateLimitConfigs.GATEWAY_BOT_INFO),
		OpenAPI({
			operationId: 'get_gateway_bot',
			summary: 'Get gateway information',
			responseSchema: GatewayBotResponse,
			statusCode: 200,
			security: [],
			tags: ['Gateway'],
			description:
				'Retrieves gateway connection information and recommended shard count for establishing WebSocket connections.',
		}),
		async (ctx) => {
			const gatewayRequestService = ctx.get('gatewayRequestService');
			return ctx.json(await gatewayRequestService.getBotGatewayInfo(ctx.req.header('Authorization') ?? null));
		},
	);
}
