// SPDX-License-Identifier: AGPL-3.0-or-later

import {stripBannerForUser} from '../infrastructure/AssetEntitlementUtils';
import type {Application} from '../models/Application';
import type {User} from '../models/User';
import {mapUserToPartialResponse} from '../user/UserMappers';
import type {ApplicationBotResponse, ApplicationResponse} from './OAuth2Types';

export function mapBotUserToResponse(
	user: User,
	opts?: {
		token?: string;
	},
): ApplicationBotResponse {
	const partial = mapUserToPartialResponse(user);
	return {
		id: partial.id,
		username: partial.username,
		discriminator: partial.discriminator,
		avatar: partial.avatar,
		banner: stripBannerForUser(user),
		bio: user.bio ?? null,
		token: opts?.token,
		mfa_enabled: (user.authenticatorTypes?.size ?? 0) > 0,
		authenticator_types: user.authenticatorTypes ? Array.from(user.authenticatorTypes) : [],
		flags: partial.flags,
	};
}

export function mapApplicationToResponse(
	application: Application,
	options?: {
		botUser?: User | null;
		botToken?: string;
		clientSecret?: string | null;
	},
): ApplicationResponse {
	const baseResponse: ApplicationResponse = {
		id: application.applicationId.toString(),
		name: application.name,
		redirect_uris: Array.from(application.oauth2RedirectUris),
		bot_public: application.botIsPublic,
		bot_require_code_grant: application.botRequireCodeGrant,
	};
	if (options?.botUser) {
		baseResponse.bot = mapBotUserToResponse(options.botUser, {token: options.botToken});
	}
	if (options?.clientSecret) {
		return {
			...baseResponse,
			client_secret: options.clientSecret,
		};
	}
	return baseResponse;
}

export function mapBotTokenResetResponse(user: User, token: string) {
	return {
		token,
		bot: mapBotUserToResponse(user),
	};
}

export function mapBotProfileToResponse(user: User) {
	const partial = mapUserToPartialResponse(user);
	return {
		id: partial.id,
		username: partial.username,
		discriminator: partial.discriminator,
		avatar: partial.avatar,
		banner: stripBannerForUser(user),
		bio: user.bio ?? null,
		flags: partial.flags,
	};
}
