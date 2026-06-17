// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	ApplicationUpdateResponse,
	ListGuildApplicationsRequest,
	ListGuildApplicationsResponse,
	ListUserApplicationsRequest,
	ListUserApplicationsResponse,
	LookupApplicationRequest,
	LookupApplicationResponse,
	TransferApplicationOwnershipRequest,
} from '@fluxer/schema/src/domains/admin/AdminApplicationSchemas';
import {requireAdminACL, requireAnyAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function ApplicationAdminController(app: HonoApp) {
	app.post(
		'/admin/applications/lookup',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.APPLICATION_LOOKUP),
		Validator('json', LookupApplicationRequest),
		OpenAPI({
			operationId: 'lookup_application',
			summary: 'Look up application',
			description:
				'Retrieves complete application details including ownership, bot user, OAuth2 redirect URIs, and credential status. Requires APPLICATION_LOOKUP permission.',
			responseSchema: LookupApplicationResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.applicationService.lookupApplication(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/applications/list-by-owner',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.APPLICATION_LIST_BY_OWNER),
		Validator('json', ListUserApplicationsRequest),
		OpenAPI({
			operationId: 'admin_list_user_applications',
			summary: 'List applications owned by a user',
			description:
				'Lists all applications (OAuth2 clients and bots) owned by a specific user. Requires APPLICATION_LIST_BY_OWNER permission.',
			responseSchema: ListUserApplicationsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.applicationService.listUserApplications(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/applications/list-by-guild',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAnyAdminACL([AdminACLs.APPLICATION_LOOKUP, AdminACLs.APPLICATION_LIST_BY_OWNER]),
		Validator('json', ListGuildApplicationsRequest),
		OpenAPI({
			operationId: 'admin_list_guild_applications',
			summary: 'List applications installed in a guild',
			description:
				'Lists OAuth2 applications whose bot users are members of a guild. Requires APPLICATION_LOOKUP or APPLICATION_LIST_BY_OWNER permission.',
			responseSchema: ListGuildApplicationsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.applicationService.listGuildApplications(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/applications/transfer-ownership',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.APPLICATION_TRANSFER_OWNERSHIP),
		Validator('json', TransferApplicationOwnershipRequest),
		OpenAPI({
			operationId: 'transfer_application_ownership',
			summary: 'Transfer application ownership',
			description:
				'Transfers application ownership to another user. Used when owner is inactive or for administrative recovery. Logged to audit log. Requires APPLICATION_TRANSFER_OWNERSHIP permission.',
			responseSchema: ApplicationUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.applicationService.transferApplicationOwnership(
					ctx.req.valid('json'),
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
}
