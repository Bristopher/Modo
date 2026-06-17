// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	ListReportsRequest,
	ListReportsResponse,
	ReportAdminResponseSchema,
	ResolveReportRequest,
	ResolveReportResponse,
	SearchReportsRequest,
	SearchReportsResponse,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {ReportIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {createReportID} from '../../BrandedTypes';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function ReportAdminController(app: HonoApp) {
	app.post(
		'/admin/reports/list',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.REPORT_VIEW),
		Validator('json', ListReportsRequest),
		OpenAPI({
			operationId: 'list_reports',
			summary: 'List reports',
			description:
				'Lists user and content reports with optional status filtering and pagination. Requires REPORT_VIEW permission.',
			responseSchema: ListReportsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserAcls = ctx.get('adminUserAcls');
			const {status, limit, offset} = ctx.req.valid('json');
			return ctx.json(await adminService.reportServiceAggregate.listReports(status ?? 0, adminUserAcls, limit, offset));
		},
	);
	app.get(
		'/admin/reports/:report_id',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.REPORT_VIEW),
		Validator('param', ReportIdParam),
		OpenAPI({
			operationId: 'get_report',
			summary: 'Get report details',
			description:
				'Retrieves detailed information about a specific report including content, reporter, and reason. Requires REPORT_VIEW permission.',
			responseSchema: ReportAdminResponseSchema,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserAcls = ctx.get('adminUserAcls');
			const {report_id} = ctx.req.valid('param');
			const report = await adminService.reportServiceAggregate.getReport(createReportID(report_id), adminUserAcls);
			return ctx.json(report);
		},
	);
	app.post(
		'/admin/reports/resolve',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.REPORT_RESOLVE),
		Validator('json', ResolveReportRequest),
		OpenAPI({
			operationId: 'resolve_report',
			summary: 'Resolve report',
			description:
				'Closes and resolves a report with optional public comment. Marks report as handled and creates audit log entry. Requires REPORT_RESOLVE permission.',
			responseSchema: ResolveReportResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const {report_id, public_comment} = ctx.req.valid('json');
			return ctx.json(
				await adminService.reportServiceAggregate.resolveReport(
					createReportID(report_id),
					adminUserId,
					public_comment || null,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/reports/search',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.REPORT_VIEW),
		Validator('json', SearchReportsRequest),
		OpenAPI({
			operationId: 'search_reports',
			summary: 'Search reports',
			description:
				'Searches and filters reports by user, content, reason, and status criteria. Supports full-text search and advanced filtering. Requires REPORT_VIEW permission.',
			responseSchema: SearchReportsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserAcls = ctx.get('adminUserAcls');
			const body = ctx.req.valid('json');
			return ctx.json(await adminService.reportServiceAggregate.searchReports(body, adminUserAcls));
		},
	);
}
