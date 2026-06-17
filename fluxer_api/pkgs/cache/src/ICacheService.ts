// SPDX-License-Identifier: AGPL-3.0-or-later

interface CacheMSetEntry<T> {
	key: string;
	value: T;
	ttlSeconds?: number;
}

export abstract class ICacheService {
	abstract get<T>(key: string): Promise<T | null>;

	abstract set<T>(key: string, value: T, ttlSeconds?: number): Promise<void>;

	abstract delete(key: string): Promise<void>;

	abstract getAndDelete<T>(key: string): Promise<T | null>;

	abstract exists(key: string): Promise<boolean>;

	abstract expire(key: string, ttlSeconds: number): Promise<void>;

	abstract ttl(key: string): Promise<number>;

	abstract mget<T>(keys: Array<string>): Promise<Array<T | null>>;

	abstract mset<T>(entries: Array<CacheMSetEntry<T>>): Promise<void>;

	abstract deletePattern(pattern: string): Promise<number>;

	abstract acquireLock(key: string, ttlSeconds: number): Promise<string | null>;

	abstract releaseLock(key: string, token: string): Promise<boolean>;

	abstract extendLock(key: string, token: string, ttlSeconds: number): Promise<boolean>;

	abstract getAndRenewTtl<T>(key: string, newTtlSeconds: number): Promise<T | null>;

	abstract publish(channel: string, message: string): Promise<void>;

	abstract sadd(key: string, member: string, ttlSeconds?: number): Promise<void>;

	abstract srem(key: string, member: string): Promise<void>;

	abstract smembers(key: string): Promise<Set<string>>;

	abstract sismember(key: string, member: string): Promise<boolean>;

	async getOrSet<T>(key: string, valueFactory: () => Promise<T>, ttlSeconds?: number): Promise<T> {
		const existingValue = await this.get<T>(key);
		if (existingValue !== null) {
			return existingValue;
		}
		const newValue = await valueFactory();
		await this.set(key, newValue, ttlSeconds);
		return newValue;
	}
}
