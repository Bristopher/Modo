// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IpInfoCache} from '@pkgs/geoip/src/IpInfoService';

const DEFAULT_HOT_TTL_SECONDS = 10 * 60;

interface TieredIpInfoCacheOptions {
	hot: IpInfoCache;
	cold: IpInfoCache;
	hotTtlSeconds?: number;
}

export function createTieredIpInfoCache(opts: TieredIpInfoCacheOptions): IpInfoCache {
	const hotTtl = opts.hotTtlSeconds ?? DEFAULT_HOT_TTL_SECONDS;
	return {
		async get<T>(key: string): Promise<T | null> {
			const hit = await opts.hot.get<T>(key).catch(() => null);
			if (hit !== null) return hit;
			const cold = await opts.cold.get<T>(key).catch(() => null);
			if (cold === null) return null;
			void opts.hot.set(key, cold, hotTtl).catch(() => {});
			return cold;
		},
		async set<T>(key: string, value: T, ttlSeconds?: number): Promise<void> {
			await Promise.all([
				opts.hot.set(key, value, hotTtl).catch(() => {}),
				opts.cold.set(key, value, ttlSeconds).catch(() => {}),
			]);
		},
	};
}
