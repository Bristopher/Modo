// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {SearchGuildsRequest} from '@fluxer/schema/src/domains/admin/AdminGuildSchemas';
import {
	GetIndexRefreshStatusRequest,
	IndexRefreshStatusResponse,
	RefreshSearchIndexRequest,
	RefreshSearchIndexResponse,
	SearchGuildsResponse,
	SearchUsersResponse,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {SearchUsersRequest} from '@fluxer/schema/src/domains/admin/AdminUserSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function SearchAdminController(app: HonoApp) {
	app.post(
		'/admin/guilds/search',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_LOOKUP),
		Validator('json', SearchGuildsRequest),
		OpenAPI({
			operationId: 'search_guilds',
			summary: 'Search guilds',
			description:
				'Searches guilds by name, ID, and other criteria. Supports full-text search and filtering. Requires GUILD_LOOKUP permission.',
			responseSchema: SearchGuildsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const body = ctx.req.valid('json');
			return ctx.json(await adminService.searchService.searchGuilds(body));
		},
	);
	app.post(
		'/admin/users/search',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.USER_LOOKUP),
		Validator('json', SearchUsersRequest),
		OpenAPI({
			operationId: 'search_users',
			summary: 'Search users',
			description:
				'Searches users by username, email, ID, last active IP, and other criteria. Supports full-text search and filtering by account status. Requires USER_LOOKUP permission.',
			responseSchema: SearchUsersResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserAcls = ctx.get('adminUserAcls');
			const body = ctx.req.valid('json');
			return ctx.json(await adminService.searchService.searchUsers(body, adminUserAcls));
		},
	);
	app.post(
		'/admin/search/refresh-index',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_LOOKUP),
		Validator('json', RefreshSearchIndexRequest),
		OpenAPI({
			operationId: 'refresh_search_index',
			summary: 'Refresh search index',
			description:
				'Trigger full or partial search index rebuild. Creates background job to reindex guilds and users. Returns job ID for status tracking. Requires GUILD_LOOKUP permission.',
			responseSchema: RefreshSearchIndexResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			return ctx.json(await adminService.searchService.refreshSearchIndex(body, adminUserId, auditLogReason));
		},
	);
	app.post(
		'/admin/search/refresh-status',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_LOOKUP),
		Validator('json', GetIndexRefreshStatusRequest),
		OpenAPI({
			operationId: 'get_search_index_refresh_status',
			summary: 'Get search index refresh status',
			description:
				'Polls status of a search index refresh job. Returns completion percentage and current phase. Requires GUILD_LOOKUP permission.',
			responseSchema: IndexRefreshStatusResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const body = ctx.req.valid('json');
			return ctx.json(await adminService.searchService.getIndexRefreshStatus(body.job_id));
		},
	);
}
