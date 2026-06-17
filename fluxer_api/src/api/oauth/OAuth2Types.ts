// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserAuthenticatorType} from '@fluxer/constants/src/UserConstants';

export interface ApplicationBotResponse {
	id: string;
	username: string;
	discriminator: string;
	avatar?: string | null;
	banner?: string | null;
	bio: string | null;
	token?: string;
	mfa_enabled?: boolean;
	authenticator_types?: Array<UserAuthenticatorType>;
	flags: number;
}

export interface ApplicationResponse {
	id: string;
	name: string;
	redirect_uris: Array<string>;
	bot_public: boolean;
	bot_require_code_grant: boolean;
	client_secret?: string;
	bot?: ApplicationBotResponse;
}
