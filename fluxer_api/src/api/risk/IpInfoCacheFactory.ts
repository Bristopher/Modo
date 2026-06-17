// SPDX-License-Identifier: AGPL-3.0-or-later

import {getDefaultCassandraClient} from '@pkgs/cassandra/src/Client';
import {createCassandraIpInfoCache} from '@pkgs/geoip/src/CassandraIpInfoCache';
import {createCassandraIpInfoRequestAuditLogger} from '@pkgs/geoip/src/CassandraIpInfoRequestAudit';
import type {IpInfoCache, IpInfoRequestAuditLogger} from '@pkgs/geoip/src/IpInfoService';
import {createPostgresIpInfoCache, createPostgresIpInfoRequestAuditLogger} from '@pkgs/geoip/src/PostgresIpInfoKv';
import {createTieredIpInfoCache} from '@pkgs/geoip/src/TieredIpInfoCache';
import {getDefaultPostgresClient} from '@pkgs/postgres/src/Client';
import {Config} from '../Config';
import {Logger} from '../Logger';

interface BuildIpInfoCacheOptions {
	hot: IpInfoCache;
}

export function buildIpInfoCache(options: BuildIpInfoCacheOptions): IpInfoCache {
	if (Config.database.backend === 'postgres') {
		return createTieredIpInfoCache({
			hot: options.hot,
			cold: createPostgresIpInfoCache({
				getClient: getDefaultPostgresClient,
				onError: (error, operation) => Logger.warn({error, operation}, 'Postgres IPInfo cache operation failed'),
			}),
		});
	}
	return createTieredIpInfoCache({
		hot: options.hot,
		cold: createCassandraIpInfoCache({getClient: getDefaultCassandraClient}),
	});
}

export function buildIpInfoRequestAuditLogger(): IpInfoRequestAuditLogger {
	if (Config.database.backend === 'postgres') {
		return createPostgresIpInfoRequestAuditLogger({
			getClient: getDefaultPostgresClient,
			onError: (error, operation) => Logger.warn({error, operation}, 'Postgres IPInfo audit operation failed'),
		});
	}
	return createCassandraIpInfoRequestAuditLogger({
		getClient: getDefaultCassandraClient,
	});
}
