// SPDX-License-Identifier: AGPL-3.0-or-later

import type {NodeSavedSession, NodeSavedState} from '@bluesky-social/oauth-client-node';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import {isJsonRecord, parseJsonWithGuard} from '../utils/JsonBoundaryUtils';

const STATE_PREFIX = 'bsky:oauth:state:';
const SESSION_PREFIX = 'bsky:oauth:session:';

function isNodeSavedState(value: unknown): value is NodeSavedState {
	return isJsonRecord(value);
}

function isNodeSavedSession(value: unknown): value is NodeSavedSession {
	return isJsonRecord(value);
}

export function createKVStateStore(kvClient: IKVProvider, ttlSeconds: number) {
	return {
		async set(key: string, internalState: NodeSavedState): Promise<void> {
			await kvClient.setex(`${STATE_PREFIX}${key}`, ttlSeconds, JSON.stringify(internalState));
		},
		async get(key: string): Promise<NodeSavedState | undefined> {
			const data = await kvClient.getdel(`${STATE_PREFIX}${key}`);
			if (!data) return undefined;
			return parseJsonWithGuard(data, isNodeSavedState) ?? undefined;
		},
		async del(key: string): Promise<void> {
			await kvClient.del(`${STATE_PREFIX}${key}`);
		},
	};
}

export function createKVSessionStore(kvClient: IKVProvider, ttlSeconds: number) {
	return {
		async set(sub: string, session: NodeSavedSession): Promise<void> {
			await kvClient.setex(`${SESSION_PREFIX}${sub}`, ttlSeconds, JSON.stringify(session));
		},
		async get(sub: string): Promise<NodeSavedSession | undefined> {
			const data = await kvClient.get(`${SESSION_PREFIX}${sub}`);
			if (!data) return undefined;
			return parseJsonWithGuard(data, isNodeSavedSession) ?? undefined;
		},
		async del(sub: string): Promise<void> {
			await kvClient.del(`${SESSION_PREFIX}${sub}`);
		},
	};
}
