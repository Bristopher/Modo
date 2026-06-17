// SPDX-License-Identifier: AGPL-3.0-or-later

import {ms} from 'itty-time';
import type {RouteRateLimitConfig} from '../middleware/RateLimitMiddleware';

export const DiscoveryRateLimitConfigs = {
	DISCOVERY_SEARCH: {
		bucket: 'discovery:search',
		config: {limit: 30, windowMs: ms('10 seconds')},
	} as RouteRateLimitConfig,
	DISCOVERY_CATEGORIES: {
		bucket: 'discovery:categories',
		config: {limit: 60, windowMs: ms('10 seconds')},
	} as RouteRateLimitConfig,
	DISCOVERY_JOIN: {
		bucket: 'discovery:join',
		config: {limit: 10, windowMs: ms('1 minute')},
	} as RouteRateLimitConfig,
	DISCOVERY_APPLY: {
		bucket: 'discovery:apply::guild_id',
		config: {limit: 5, windowMs: ms('1 minute')},
	} as RouteRateLimitConfig,
	DISCOVERY_STATUS: {
		bucket: 'discovery:status::guild_id',
		config: {limit: 30, windowMs: ms('10 seconds')},
	} as RouteRateLimitConfig,
	DISCOVERY_ADMIN_LIST: {
		bucket: 'discovery:admin:list',
		config: {limit: 30, windowMs: ms('10 seconds')},
	} as RouteRateLimitConfig,
	DISCOVERY_ADMIN_ACTION: {
		bucket: 'discovery:admin:action',
		config: {limit: 20, windowMs: ms('10 seconds')},
	} as RouteRateLimitConfig,
} as const;
