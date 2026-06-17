// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {MissingACLError} from '@fluxer/errors/src/domains/core/MissingACLError';
import {
	AdminArchiveResponseSchema,
	DownloadUrlResponseSchema,
	GetArchiveResponseSchema,
	ListArchivesRequest,
	ListArchivesResponseSchema,
	TriggerGuildArchiveRequest,
	TriggerUserArchiveRequest,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {ArchivePathParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {createGuildID, createUserID} from '../../BrandedTypes';
import {requireAdminACL, requireAnyAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

function canViewArchive(adminAcls: Set<string>, subjectType: 'user' | 'guild'): boolean {
	if (adminAcls.has(AdminACLs.WILDCARD) || adminAcls.has(AdminACLs.ARCHIVE_VIEW_ALL)) return true;
	if (subjectType === 'user') return adminAcls.has(AdminACLs.ARCHIVE_TRIGGER_USER);
	return adminAcls.has(AdminACLs.ARCHIVE_TRIGGER_GUILD);
}

export function ArchiveAdminController(app: HonoApp) {
	app.post(
		'/admin/archives/user',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.ARCHIVE_TRIGGER_USER),
		Validator('json', TriggerUserArchiveRequest),
		OpenAPI({
			operationId: 'trigger_user_archive',
			summary: 'Trigger user archive',
			responseSchema: AdminArchiveResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				"Initiates a data export for a user. Creates an archive containing all the user's data (messages, server memberships, preferences, etc.) for export or compliance purposes.",
		}),
		async (ctx) => {
			const adminArchiveService = ctx.get('adminArchiveService');
			const adminUserId = ctx.get('adminUserId');
			const body = ctx.req.valid('json');
			const result = await adminArchiveService.triggerUserArchive(
				createUserID(body.user_id),
				adminUserId,
				body.include_attachments,
			);
			return ctx.json(result, 200);
		},
	);
	app.post(
		'/admin/archives/guild',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.ARCHIVE_TRIGGER_GUILD),
		Validator('json', TriggerGuildArchiveRequest),
		OpenAPI({
			operationId: 'trigger_guild_archive',
			summary: 'Trigger guild archive',
			responseSchema: AdminArchiveResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Initiates a data export for a guild (server). Creates an archive containing all guild data including channels, messages, members, roles, and settings.',
		}),
		async (ctx) => {
			const adminArchiveService = ctx.get('adminArchiveService');
			const adminUserId = ctx.get('adminUserId');
			const body = ctx.req.valid('json');
			const result = await adminArchiveService.triggerGuildArchive(
				createGuildID(body.guild_id),
				adminUserId,
				body.include_attachments,
			);
			return ctx.json(result, 200);
		},
	);
	app.post(
		'/admin/archives/list',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAnyAdminACL([AdminACLs.ARCHIVE_VIEW_ALL, AdminACLs.ARCHIVE_TRIGGER_USER, AdminACLs.ARCHIVE_TRIGGER_GUILD]),
		Validator('json', ListArchivesRequest),
		OpenAPI({
			operationId: 'list_archives',
			summary: 'List archives',
			responseSchema: ListArchivesResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Query and filter created archives by type (user or guild), subject ID, requestor, and expiration status. Admins with limited ACLs see only archives matching their permissions.',
		}),
		async (ctx) => {
			const adminArchiveService = ctx.get('adminArchiveService');
			const adminAcls = ctx.get('adminUserAcls');
			const body = ctx.req.valid('json') as ListArchivesRequest;
			if (
				body.subject_type === 'all' &&
				!adminAcls.has(AdminACLs.ARCHIVE_VIEW_ALL) &&
				!adminAcls.has(AdminACLs.WILDCARD)
			) {
				throw new MissingACLError(AdminACLs.ARCHIVE_VIEW_ALL);
			}
			if (
				body.subject_type !== 'all' &&
				!canViewArchive(adminAcls, body.subject_type) &&
				!adminAcls.has(AdminACLs.WILDCARD)
			) {
				throw new MissingACLError(
					body.subject_type === 'user' ? AdminACLs.ARCHIVE_TRIGGER_USER : AdminACLs.ARCHIVE_TRIGGER_GUILD,
				);
			}
			const result = await adminArchiveService.listArchives({
				subjectType: body.subject_type as 'user' | 'guild' | 'all',
				subjectId: body.subject_id ?? undefined,
				requestedBy: body.requested_by ?? undefined,
				limit: body.limit,
				includeExpired: body.include_expired,
			});
			return ctx.json({archives: result}, 200);
		},
	);
	app.get(
		'/admin/archives/:subjectType/:subjectId/:archiveId',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAnyAdminACL([AdminACLs.ARCHIVE_VIEW_ALL, AdminACLs.ARCHIVE_TRIGGER_USER, AdminACLs.ARCHIVE_TRIGGER_GUILD]),
		Validator('param', ArchivePathParam),
		OpenAPI({
			operationId: 'get_archive_details',
			summary: 'Get archive details',
			responseSchema: GetArchiveResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Retrieve metadata for a specific archive including its status, creation time, expiration, and file location. Does not return the archive contents themselves.',
		}),
		async (ctx) => {
			const adminArchiveService = ctx.get('adminArchiveService');
			const adminAcls = ctx.get('adminUserAcls');
			const params = ctx.req.valid('param');
			const subjectType = params.subjectType;
			if (!canViewArchive(adminAcls, subjectType) && !adminAcls.has(AdminACLs.WILDCARD)) {
				throw new MissingACLError(
					subjectType === 'user' ? AdminACLs.ARCHIVE_TRIGGER_USER : AdminACLs.ARCHIVE_TRIGGER_GUILD,
				);
			}
			const subjectId = params.subjectId;
			const archiveId = params.archiveId;
			const archive = await adminArchiveService.getArchive(subjectType, subjectId, archiveId);
			return ctx.json({archive}, 200);
		},
	);
	app.get(
		'/admin/archives/:subjectType/:subjectId/:archiveId/download',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAnyAdminACL([AdminACLs.ARCHIVE_VIEW_ALL, AdminACLs.ARCHIVE_TRIGGER_USER, AdminACLs.ARCHIVE_TRIGGER_GUILD]),
		Validator('param', ArchivePathParam),
		OpenAPI({
			operationId: 'get_archive_download_url',
			summary: 'Get archive download URL',
			responseSchema: DownloadUrlResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Generate a time-limited download link to the archive file. The URL provides direct access to download the compressed archive contents.',
		}),
		async (ctx) => {
			const adminArchiveService = ctx.get('adminArchiveService');
			const adminAcls = ctx.get('adminUserAcls');
			const params = ctx.req.valid('param');
			const subjectType = params.subjectType;
			if (!canViewArchive(adminAcls, subjectType) && !adminAcls.has(AdminACLs.WILDCARD)) {
				throw new MissingACLError(
					subjectType === 'user' ? AdminACLs.ARCHIVE_TRIGGER_USER : AdminACLs.ARCHIVE_TRIGGER_GUILD,
				);
			}
			const subjectId = params.subjectId;
			const archiveId = params.archiveId;
			const result = await adminArchiveService.getDownloadUrl(subjectType, subjectId, archiveId);
			return ctx.json(result, 200);
		},
	);
}
