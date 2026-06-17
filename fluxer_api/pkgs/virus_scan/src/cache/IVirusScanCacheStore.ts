// SPDX-License-Identifier: AGPL-3.0-or-later

export interface IVirusScanCacheStore {
	get<T>(key: string): Promise<T | null>;
	set<T>(key: string, value: T, ttlSeconds?: number): Promise<void>;
}
