// SPDX-License-Identifier: AGPL-3.0-or-later

import {ms} from 'itty-time';
import type {RouteRateLimitConfig} from '../middleware/RateLimitMiddleware';

export const DonationRateLimitConfigs = {
	DONATION_REQUEST_LINK: {
		bucket: 'donation:request_link',
		config: {limit: 3, windowMs: ms('1 hour')},
	} as RouteRateLimitConfig,
	DONATION_MANAGE: {
		bucket: 'donation:manage',
		config: {limit: 10, windowMs: ms('1 minute')},
	} as RouteRateLimitConfig,
	DONATION_CHECKOUT: {
		bucket: 'donation:checkout',
		config: {limit: 5, windowMs: ms('1 minute')},
	} as RouteRateLimitConfig,
} as const;
