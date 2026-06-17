// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {SendSystemDmRequest, SendSystemDmResponse} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function SystemDmAdminController(app: HonoApp) {
	app.post(
		'/admin/system-dm/send',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_MESSAGE_OPERATION),
		requireAdminACL(AdminACLs.SYSTEM_DM_SEND),
		Validator('json', SendSystemDmRequest),
		OpenAPI({
			operationId: 'send_system_dm',
			summary: 'Send system DM',
			responseSchema: SendSystemDmResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Queue a worker job that sends the same system DM content to each provided user ID. Progress is observable via the Jobs admin page (task_type=sendSystemDm). Requires SYSTEM_DM_SEND permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const payload = ctx.req.valid('json');
			const result = await adminService.sendSystemDm(
				{content: payload.content, userIds: payload.user_ids.map((id) => id.toString())},
				adminUserId,
				auditLogReason,
			);
			return ctx.json(result);
		},
	);
}
