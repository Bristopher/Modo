// SPDX-License-Identifier: AGPL-3.0-or-later

import {DEFAULT_KV_TIMEOUT_MS} from '@fluxer/constants/src/Timeouts';

export interface IKVLogger {
	debug(obj: object, msg?: string): void;
	error(obj: object, msg?: string): void;
}

export type KVClientMode = 'standalone' | 'cluster';

export interface KVClusterNode {
	host: string;
	port: number;
}

export interface KVClientConfig {
	url: string;
	mode?: KVClientMode;
	clusterNodes?: Array<KVClusterNode>;
	clusterNatMap?: Record<string, KVClusterNode>;
	timeoutMs?: number;
	logger?: IKVLogger;
}

export interface ResolvedKVClientConfig {
	url: string;
	mode: KVClientMode;
	clusterNodes: Array<KVClusterNode>;
	clusterNatMap: Record<string, KVClusterNode>;
	timeoutMs: number;
	logger: IKVLogger;
}

const noopLogger: IKVLogger = {
	debug() {},
	error() {},
};

export function resolveKVClientConfig(config: KVClientConfig | string): ResolvedKVClientConfig {
	if (typeof config === 'string') {
		return {
			url: normalizeUrl(config),
			mode: 'standalone' as const,
			clusterNodes: [],
			clusterNatMap: {},
			timeoutMs: DEFAULT_KV_TIMEOUT_MS,
			logger: noopLogger,
		};
	}
	return {
		url: normalizeUrl(config.url),
		mode: config.mode ?? 'standalone',
		clusterNodes: config.clusterNodes ?? [],
		clusterNatMap: config.clusterNatMap ?? {},
		timeoutMs: config.timeoutMs ?? DEFAULT_KV_TIMEOUT_MS,
		logger: config.logger ?? noopLogger,
	};
}

function normalizeUrl(url: string): string {
	const trimmed = url.trim();
	if (trimmed.length === 0) {
		throw new Error('KV client URL must not be empty');
	}
	return trimmed;
}
