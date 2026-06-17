// SPDX-License-Identifier: AGPL-3.0-or-later

import {Client} from '@elastic/elasticsearch';

export interface ElasticsearchClientConfig {
	node: string;
	auth?: {
		apiKey?: string;
		username?: string;
		password?: string;
	};
	requestTimeoutMs: number;
	tlsRejectUnauthorized?: boolean;
}

export function createElasticsearchClient(config: ElasticsearchClientConfig): Client {
	return new Client({
		node: config.node,
		auth: config.auth?.apiKey
			? {apiKey: config.auth.apiKey}
			: config.auth?.username
				? {username: config.auth.username, password: config.auth.password ?? ''}
				: undefined,
		requestTimeout: config.requestTimeoutMs,
		tls: config.tlsRejectUnauthorized === false ? {rejectUnauthorized: false} : undefined,
	});
}
