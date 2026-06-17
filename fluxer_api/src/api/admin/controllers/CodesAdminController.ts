// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {FeatureNotAvailableSelfHostedError} from '@fluxer/errors/src/domains/core/FeatureNotAvailableSelfHostedError';
import {CodesResponse, GenerateGiftCodesRequest} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {Config} from '../../Config';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

function trimTrailingSlash(value: string): string {
	return value.endsWith('/') ? value.slice(0, -1) : value;
}

export function CodesAdminController(app: HonoApp) {
	app.post(
		'/admin/codes/gift',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_CODE_GENERATION),
		requireAdminACL(AdminACLs.GIFT_CODES_GENERATE),
		Validator('json', GenerateGiftCodesRequest),
		OpenAPI({
			operationId: 'generate_gift_codes',
			summary: 'Generate gift codes',
			description:
				'Create one-use Plutonium gift codes with an explicit positive duration. Lifetime gifts are not supported.',
			responseSchema: CodesResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			if (Config.instance.selfHosted) {
				throw new FeatureNotAvailableSelfHostedError();
			}
			const {count, duration_type, duration_quantity} = ctx.req.valid('json');
			const codes = await ctx.get('adminService').codeGenerationService.generateGiftCodes({
				count,
				durationType: duration_type,
				durationQuantity: duration_quantity,
			});
			const baseUrl = trimTrailingSlash(Config.endpoints.gift);
			return ctx.json({
				codes: codes.map((code) => `${baseUrl}/${code}`),
			});
		},
	);
}
