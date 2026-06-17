// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IVirusHashCache} from '@pkgs/virus_scan/src/cache/IVirusHashCache';
import type {IVirusScanCacheStore} from '@pkgs/virus_scan/src/cache/IVirusScanCacheStore';
import {seconds} from 'itty-time';

interface VirusHashCacheConfig {
	keyPrefix?: string;
	ttlSeconds?: number;
}

export class VirusHashCache implements IVirusHashCache {
	private readonly keyPrefix: string;
	private readonly ttlSeconds: number;

	constructor(
		private cacheStore: IVirusScanCacheStore,
		config: VirusHashCacheConfig = {},
	) {
		this.keyPrefix = config.keyPrefix ?? 'virus';
		this.ttlSeconds = config.ttlSeconds ?? seconds('7 days');
	}

	async isKnownVirusHash(fileHash: string): Promise<boolean> {
		const cachedValue = await this.cacheStore.get(this.buildCacheKey(fileHash));
		return cachedValue != null;
	}

	async cacheVirusHash(fileHash: string): Promise<void> {
		await this.cacheStore.set(this.buildCacheKey(fileHash), 'true', this.ttlSeconds);
	}

	private buildCacheKey(fileHash: string): string {
		return `${this.keyPrefix}:${fileHash}`;
	}
}
