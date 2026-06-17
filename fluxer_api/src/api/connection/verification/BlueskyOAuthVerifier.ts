// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IBlueskyOAuthService} from '../../bluesky/IBlueskyOAuthService';
import {Logger} from '../../Logger';
import {BlueskyOAuthNotEnabledError} from '../errors/BlueskyOAuthNotEnabledError';
import type {ConnectionVerificationParams, IConnectionVerifier} from './IConnectionVerifier';

export class BlueskyOAuthVerifier implements IConnectionVerifier {
	constructor(private readonly oauthService: IBlueskyOAuthService) {}

	async verify(params: ConnectionVerificationParams): Promise<boolean> {
		try {
			const result = await this.oauthService.restoreAndVerify(params.identifier);
			return result !== null;
		} catch (error) {
			if (error instanceof BlueskyOAuthNotEnabledError) {
				throw error;
			}
			Logger.error(
				{
					identifier: params.identifier,
					error: error instanceof Error ? error.message : String(error),
				},
				'Failed to verify Bluesky connection',
			);
			return false;
		}
	}
}
