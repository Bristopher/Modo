// SPDX-License-Identifier: AGPL-3.0-or-later

import {SsoRequiredError} from '@fluxer/errors/src/domains/auth/SsoRequiredError';
import {createMiddleware} from 'hono/factory';
import type {HonoEnv} from '../types/HonoEnv';

export const LocalAuthMiddleware = createMiddleware<HonoEnv>(async (ctx, next) => {
	const ssoService = ctx.get('ssoService');
	if (await ssoService.isEnforced()) {
		throw new SsoRequiredError();
	}
	await next();
});
