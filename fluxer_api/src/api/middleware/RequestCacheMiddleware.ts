// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserPartialResponse} from '@fluxer/schema/src/domains/user/UserResponseSchemas';
import {createMiddleware} from 'hono/factory';
import type {HonoEnv} from '../types/HonoEnv';

export interface RequestCache {
	userPartials: Map<bigint, UserPartialResponse>;
	messageMentionChannels: Map<string, Array<{id: string; name: string; type: number}>>;
	clear(): void;
}

class RequestCacheImpl implements RequestCache {
	userPartials = new Map<bigint, UserPartialResponse>();
	messageMentionChannels = new Map<string, Array<{id: string; name: string; type: number}>>();

	clear(): void {
		this.userPartials.clear();
		this.messageMentionChannels.clear();
	}
}

export const RequestCacheMiddleware = createMiddleware<HonoEnv>(async (ctx, next) => {
	const requestCache: RequestCache = new RequestCacheImpl();
	ctx.set('requestCache', requestCache);
	try {
		await next();
	} finally {
		requestCache.clear();
	}
});

export function createRequestCache(): RequestCache {
	return new RequestCacheImpl();
}
