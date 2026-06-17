// SPDX-License-Identifier: AGPL-3.0-or-later

import {readFile} from 'node:fs/promises';
import {isAbsolute} from 'node:path';
import {Agent} from '@atproto/api';
import {JoseKey} from '@bluesky-social/jwk-jose';
import {NodeOAuthClient, requestLocalLock} from '@bluesky-social/oauth-client-node';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import {createUserID, type UserID} from '../BrandedTypes';
import type {BlueskyOAuthConfig} from '../config/APIConfig';
import {Logger} from '../Logger';
import {parseJsonRecord} from '../utils/JsonBoundaryUtils';
import {createKVSessionStore, createKVStateStore} from './BlueskyOAuthStores';
import type {BlueskyAuthorizeResult, BlueskyCallbackResult, IBlueskyOAuthService} from './IBlueskyOAuthService';

export class BlueskyOAuthService implements IBlueskyOAuthService {
	private client: NodeOAuthClient;

	private constructor(client: NodeOAuthClient) {
		this.client = client;
	}

	static async create(
		config: BlueskyOAuthConfig,
		kvClient: IKVProvider,
		apiPublicEndpoint: string,
	): Promise<BlueskyOAuthService> {
		const baseUrl = apiPublicEndpoint.replace(/\/$/, '');
		const keyset = await Promise.all(
			config.keys.map(async (key) => {
				const keyPath = key.private_key_path;
				if (!key.private_key && keyPath && !isAbsolute(keyPath)) {
					throw new Error(`Bluesky OAuth key path must be absolute, got: ${keyPath}`);
				}
				const keyData = key.private_key ?? (await readFile(keyPath!, 'utf-8'));
				return JoseKey.fromImportable(keyData, key.kid);
			}),
		);
		const stateStore = createKVStateStore(kvClient, 3600);
		const sessionStore = createKVSessionStore(kvClient, 86400);
		const client = new NodeOAuthClient({
			clientMetadata: {
				client_id: `${baseUrl}/connections/bluesky/client-metadata.json`,
				client_name: config.client_name,
				client_uri: baseUrl,
				logo_uri: config.logo_uri || undefined,
				tos_uri: config.tos_uri || undefined,
				policy_uri: config.policy_uri || undefined,
				redirect_uris: [`${baseUrl}/connections/bluesky/callback`],
				grant_types: ['authorization_code', 'refresh_token'],
				scope: 'atproto',
				response_types: ['code'],
				application_type: 'web',
				token_endpoint_auth_method: 'private_key_jwt',
				token_endpoint_auth_signing_alg: 'ES256',
				dpop_bound_access_tokens: true,
				jwks_uri: `${baseUrl}/connections/bluesky/jwks.json`,
			},
			keyset,
			requestLock: requestLocalLock,
			stateStore,
			sessionStore,
		});
		return new BlueskyOAuthService(client);
	}

	get clientMetadata(): Record<string, unknown> {
		return this.client.clientMetadata as Record<string, unknown>;
	}

	get jwks(): Record<string, unknown> {
		return this.client.jwks as Record<string, unknown>;
	}

	async authorize(handle: string, userId: UserID): Promise<BlueskyAuthorizeResult> {
		const statePayload = JSON.stringify({userId: String(userId)});
		const url = await this.client.authorize(handle, {state: statePayload});
		return {authorizeUrl: url.toString()};
	}

	async callback(params: URLSearchParams): Promise<BlueskyCallbackResult> {
		const {session, state} = await this.client.callback(params);
		const parsed = state ? parseJsonRecord(state) : null;
		const parsedUserId = typeof parsed?.userId === 'string' ? parsed.userId : null;
		if (!parsedUserId) {
			throw new Error('Bluesky OAuth callback state is missing userId');
		}
		const userId = createUserID(BigInt(parsedUserId));
		const handle = await this.resolveHandle(session.did);
		return {
			userId,
			did: session.did,
			handle,
		};
	}

	async restoreAndVerify(did: string): Promise<{
		handle: string;
	} | null> {
		try {
			await this.client.restore(did);
			const handle = await this.resolveHandle(did);
			return {handle};
		} catch (error) {
			Logger.error(
				{
					did,
					error: error instanceof Error ? error.message : String(error),
				},
				'Failed to restore and verify Bluesky session',
			);
			return null;
		}
	}

	private async resolveHandle(did: string): Promise<string> {
		const agent = new Agent('https://public.api.bsky.app');
		const profile = await agent.getProfile({actor: did});
		return profile.data.handle;
	}

	async revoke(did: string): Promise<void> {
		try {
			const session = await this.client.restore(did);
			await session.signOut();
		} catch (error) {
			Logger.debug(
				{
					did,
					error: error instanceof Error ? error.message : String(error),
				},
				'Failed to revoke Bluesky session (session may be expired or already revoked)',
			);
		}
	}
}
