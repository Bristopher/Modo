// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ICassandraClient} from '@pkgs/cassandra/src/Client';
import type {IpInfoCache} from '@pkgs/geoip/src/IpInfoService';

const TABLE = 'ipinfo_cache';
const SELECT_CQL = `SELECT payload FROM ${TABLE} WHERE cache_key = :cache_key LIMIT 1;`;
const INSERT_WITH_TTL_CQL = `INSERT INTO ${TABLE} (cache_key, payload) VALUES (:cache_key, :payload) USING TTL :ttl;`;
const INSERT_DEFAULT_TTL_CQL = `INSERT INTO ${TABLE} (cache_key, payload) VALUES (:cache_key, :payload);`;

interface CassandraIpInfoCacheOptions {
	client?: ICassandraClient;
	getClient?: () => ICassandraClient;
}

export function createCassandraIpInfoCache(options: CassandraIpInfoCacheOptions): IpInfoCache {
	return {
		async get<T>(key: string): Promise<T | null> {
			try {
				const client = options.client ?? options.getClient?.();
				if (!client) {
					return null;
				}
				const result = await client.execute({cql: SELECT_CQL, params: {cache_key: key}});
				const row = result.first();
				if (!row) return null;
				const payload = row.get('payload');
				if (typeof payload !== 'string') return null;
				return JSON.parse(payload) as T;
			} catch {
				return null;
			}
		},
		async set<T>(key: string, value: T, ttlSeconds?: number): Promise<void> {
			let payload: string;
			try {
				payload = JSON.stringify(value);
			} catch {
				return;
			}
			try {
				const client = options.client ?? options.getClient?.();
				if (!client) {
					return;
				}
				if (ttlSeconds != null && Number.isFinite(ttlSeconds) && ttlSeconds > 0) {
					await client.execute({
						cql: INSERT_WITH_TTL_CQL,
						params: {cache_key: key, payload, ttl: ttlSeconds},
					});
				} else {
					await client.execute({
						cql: INSERT_DEFAULT_TTL_CQL,
						params: {cache_key: key, payload},
					});
				}
			} catch {}
		},
	};
}
