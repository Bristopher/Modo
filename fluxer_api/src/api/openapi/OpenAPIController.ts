// SPDX-License-Identifier: AGPL-3.0-or-later

import * as fs from 'node:fs';
import * as path from 'node:path';
import {fileURLToPath} from 'node:url';
import type {Hono} from 'hono';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoEnv} from '../types/HonoEnv';

const SPEC_PATH = path.join(path.dirname(fileURLToPath(import.meta.url)), 'openapi.json');
const SPEC_BODY = fs.readFileSync(SPEC_PATH, 'utf-8');

export function OpenAPIController(app: Hono<HonoEnv>): void {
	app.get('/openapi.json', RateLimitMiddleware(RateLimitConfigs.INSTANCE_INFO), (ctx) => {
		ctx.header('Access-Control-Allow-Origin', '*');
		ctx.header('Content-Type', 'application/json; charset=utf-8');
		return ctx.body(SPEC_BODY);
	});
}
