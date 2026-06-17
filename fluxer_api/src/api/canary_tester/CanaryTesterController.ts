// SPDX-License-Identifier: AGPL-3.0-or-later

import {CANARY_TESTER_MIN_ACCOUNT_AGE_MS, CANARY_TESTERS_GUILD_ID} from '@fluxer/constants/src/AppConstants';
import {JoinSourceTypes} from '@fluxer/constants/src/GuildConstants';
import {CanaryTesterEmailVerificationRequiredError} from '@fluxer/errors/src/domains/auth/EmailVerificationRequiredError';
import {AccountTooNewForGuildError} from '@fluxer/errors/src/domains/guild/AccountTooNewForGuildError';
import {AccountSuspiciousActivityError} from '@fluxer/errors/src/domains/user/AccountSuspiciousActivityError';
import {SuccessResponse} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {extractTimestampFromSnowflakeAsDateBigInt} from '@fluxer/snowflake/src/SnowflakeUtils';
import {createGuildID} from '../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../middleware/RateLimitMiddleware';
import {OpenAPI} from '../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../RateLimitConfig';
import type {HonoApp} from '../types/HonoEnv';
import {getEffectiveSuspiciousFlags} from '../user/UserHelpers';

export function CanaryTesterController(app: HonoApp) {
	app.post(
		'/users/@me/canary-tester/join',
		RateLimitMiddleware(RateLimitConfigs.USER_CANARY_TESTER_JOIN),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'join_canary_testers',
			summary: 'Join the canary testers guild',
			description:
				'Adds the authenticated user to the hardcoded Fluxer Testers guild used for canary feedback. Restricted to non-bot users with verified email, an account at least 30 minutes old, no effective suspicious-activity flags, and not banned from the target guild. Rate-limited; surfaced via the canary nagbar.',
			responseSchema: SuccessResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Users'],
		}),
		async (ctx) => {
			const user = ctx.get('user');
			const userId = user.id;
			if (!user.email || !user.emailVerified) {
				throw new CanaryTesterEmailVerificationRequiredError();
			}
			const accountCreatedAt = extractTimestampFromSnowflakeAsDateBigInt(BigInt(userId.toString()));
			if (Date.now() - accountCreatedAt.getTime() < CANARY_TESTER_MIN_ACCOUNT_AGE_MS) {
				throw new AccountTooNewForGuildError();
			}
			const effectiveFlags = getEffectiveSuspiciousFlags(user);
			if (effectiveFlags !== 0) {
				throw new AccountSuspiciousActivityError(effectiveFlags);
			}
			const guildService = ctx.get('guildService');
			const requestCache = ctx.get('requestCache');
			const guildId = createGuildID(BigInt(CANARY_TESTERS_GUILD_ID));
			await guildService.members.addUserToGuild({
				userId,
				guildId,
				sendJoinMessage: true,
				requestCache,
				joinSourceType: JoinSourceTypes.INSTANT_INVITE,
			});
			return ctx.json({success: true} satisfies SuccessResponse);
		},
	);
}
