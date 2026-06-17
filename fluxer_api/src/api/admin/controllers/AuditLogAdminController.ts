// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	AuditLogsListResponseSchema,
	ListAuditLogsRequest,
	SearchAuditLogsRequest,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function AuditLogAdminController(app: HonoApp) {
	app.post(
		'/admin/audit-logs',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_AUDIT_LOG),
		requireAdminACL(AdminACLs.AUDIT_LOG_VIEW),
		Validator('json', ListAuditLogsRequest),
		OpenAPI({
			operationId: 'list_audit_logs',
			summary: 'List audit logs',
			responseSchema: AuditLogsListResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Retrieve a paginated list of audit logs with optional filtering by date range, action type, or actor. Used for tracking administrative operations and compliance auditing.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.auditService.listAuditLogs(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/audit-logs/search',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_AUDIT_LOG),
		requireAdminACL(AdminACLs.AUDIT_LOG_VIEW),
		Validator('json', SearchAuditLogsRequest),
		OpenAPI({
			operationId: 'search_audit_logs',
			summary: 'Search audit logs',
			responseSchema: AuditLogsListResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Perform a full-text search across audit logs for specific events or changes. Allows targeted queries for compliance investigations or incident response.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.auditService.searchAuditLogs(ctx.req.valid('json')));
		},
	);
}
