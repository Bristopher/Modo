// SPDX-License-Identifier: AGPL-3.0-or-later

import {AccessDeniedError} from '@fluxer/errors/src/domains/core/AccessDeniedError';
import {UnauthorizedError} from '@fluxer/errors/src/domains/core/UnauthorizedError';
import {AccountSuspiciousActivityError} from '@fluxer/errors/src/domains/user/AccountSuspiciousActivityError';
import {createMiddleware} from 'hono/factory';
import type {HonoEnv} from '../types/HonoEnv';
import {getEffectiveSuspiciousFlags} from '../user/UserHelpers';

function ensureOAuth2BearerRouteSupport(
	authTokenType: 'session' | 'bearer' | 'bot' | 'admin_api_key' | undefined,
	oauthBearerAllowed: boolean | undefined,
): void {
	if (authTokenType === 'bearer' && !oauthBearerAllowed) {
		throw new AccessDeniedError();
	}
}

export const LoginRequired = createMiddleware<HonoEnv>(async (ctx, next) => {
	const user = ctx.get('user');
	if (!user) {
		throw new UnauthorizedError();
	}
	ensureOAuth2BearerRouteSupport(ctx.get('authTokenType'), ctx.get('oauthBearerAllowed'));
	const effectiveFlags = getEffectiveSuspiciousFlags(user);
	if (effectiveFlags !== 0) {
		throw new AccountSuspiciousActivityError(effectiveFlags);
	}
	await next();
});
export const LoginRequiredAllowSuspicious = createMiddleware<HonoEnv>(async (ctx, next) => {
	const user = ctx.get('user');
	if (!user) {
		throw new UnauthorizedError();
	}
	ensureOAuth2BearerRouteSupport(ctx.get('authTokenType'), ctx.get('oauthBearerAllowed'));
	await next();
});
export const DefaultUserOnly = createMiddleware<HonoEnv>(async (ctx, next) => {
	const user = ctx.get('user');
	if (!user) {
		throw new UnauthorizedError();
	}
	if (user.isBot) {
		throw new AccessDeniedError();
	}
	await next();
});
