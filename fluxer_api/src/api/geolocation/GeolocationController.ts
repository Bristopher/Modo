// SPDX-License-Identifier: AGPL-3.0-or-later

import {resolveClientGeoip} from '@pkgs/geoip/src/ResolveClientGeoip';
import type {Hono} from 'hono';
import {Config} from '../Config';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoEnv} from '../types/HonoEnv';

export function GeolocationController(app: Hono<HonoEnv>): void {
	app.get('/ip', RateLimitMiddleware(RateLimitConfigs.IP_GEO_LOOKUP), async (ctx) => {
		ctx.header('Access-Control-Allow-Origin', '*');
		ctx.header('Access-Control-Allow-Headers', 'Content-Type');
		ctx.header('Access-Control-Allow-Methods', 'GET, OPTIONS');
		const response = await resolveClientGeoip(ctx.req.raw, {
			maxmindDbPath: Config.geoip.maxmindDbPath,
			trustClientIpHeader: Config.proxy.trust_client_ip_header,
			clientIpHeaderName: Config.proxy.client_ip_header,
		});
		return ctx.json(response);
	});
	app.options('/ip', (ctx) => {
		ctx.header('Access-Control-Allow-Origin', '*');
		ctx.header('Access-Control-Allow-Headers', 'Content-Type');
		ctx.header('Access-Control-Allow-Methods', 'GET, OPTIONS');
		return ctx.body(null, 204);
	});
}
