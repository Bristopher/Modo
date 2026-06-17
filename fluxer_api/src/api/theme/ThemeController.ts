// SPDX-License-Identifier: AGPL-3.0-or-later

import {ThemeCreateRequest, ThemeCreateResponse} from '@fluxer/schema/src/domains/theme/ThemeSchemas';
import {DefaultUserOnly, LoginRequired} from '../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {OpenAPI} from '../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoApp} from '../types/HonoEnv';
import {Validator} from '../Validator';

export function ThemeController(app: HonoApp) {
	app.post(
		'/users/@me/themes',
		RateLimitMiddleware(RateLimitConfigs.THEME_SHARE_CREATE),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'create_theme',
			summary: 'Create theme',
			responseSchema: ThemeCreateResponse,
			statusCode: 201,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Themes'],
			description: 'Creates a new custom theme with CSS styling that can be shared with other users.',
		}),
		Validator('json', ThemeCreateRequest),
		async (ctx) => {
			const {css} = ctx.req.valid('json');
			const theme = await ctx.get('themeService').createTheme(css);
			return ctx.json(theme, 201);
		},
	);
}
