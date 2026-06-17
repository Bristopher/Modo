// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {PurgeGuildAssetsRequest, PurgeGuildAssetsResponseSchema} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {AdminRateLimitConfigs} from '../../rate_limit_configs/AdminRateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function AssetAdminController(app: HonoApp) {
	app.post(
		'/admin/assets/purge',
		RateLimitMiddleware(AdminRateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.ASSET_PURGE),
		Validator('json', PurgeGuildAssetsRequest),
		OpenAPI({
			operationId: 'purge_guild_assets',
			summary: 'Purge guild assets',
			responseSchema: PurgeGuildAssetsResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Delete and clean up all assets belonging to a guild, including icons, banners, and other media. This is a destructive operation used for cleanup during guild management or compliance actions.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const data = ctx.req.valid('json');
			return ctx.json(
				await adminService.assetPurgeService.purgeGuildAssets({
					ids: data.ids,
					adminUserId,
					auditLogReason,
				}),
			);
		},
	);
}
