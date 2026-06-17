// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';

export interface BlueskyAuthorizeResult {
	authorizeUrl: string;
}

export interface BlueskyCallbackResult {
	userId: UserID;
	did: string;
	handle: string;
}

export interface IBlueskyOAuthService {
	readonly clientMetadata: Record<string, unknown>;
	readonly jwks: Record<string, unknown>;
	authorize(handle: string, userId: UserID): Promise<BlueskyAuthorizeResult>;
	callback(params: URLSearchParams): Promise<BlueskyCallbackResult>;
	restoreAndVerify(did: string): Promise<{
		handle: string;
	} | null>;
	revoke(did: string): Promise<void>;
}
