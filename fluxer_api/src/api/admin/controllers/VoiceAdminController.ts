// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	CreateVoiceRegionRequest,
	CreateVoiceRegionResponse,
	CreateVoiceServerRequest,
	CreateVoiceServerResponse,
	DeleteVoiceRegionRequest,
	DeleteVoiceResponse,
	DeleteVoiceServerRequest,
	GetVoiceRegionRequest,
	GetVoiceRegionResponse,
	GetVoiceServerRequest,
	GetVoiceServerResponse,
	ListVoiceRegionsRequest,
	ListVoiceRegionsResponse,
	ListVoiceServersRequest,
	ListVoiceServersResponse,
	UpdateVoiceRegionRequest,
	UpdateVoiceRegionResponse,
	UpdateVoiceServerRequest,
	UpdateVoiceServerResponse,
} from '@fluxer/schema/src/domains/admin/AdminVoiceSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function VoiceAdminController(app: HonoApp) {
	app.post(
		'/admin/voice/regions/list',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_REGION_LIST),
		Validator('json', ListVoiceRegionsRequest),
		OpenAPI({
			operationId: 'list_voice_regions',
			summary: 'List voice regions',
			responseSchema: ListVoiceRegionsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Lists all configured voice server regions with status and server count. Shows region names, latency info, and availability. Requires VOICE_REGION_LIST permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.voiceService.listVoiceRegions(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/voice/regions/get',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_REGION_LIST),
		Validator('json', GetVoiceRegionRequest),
		OpenAPI({
			operationId: 'get_voice_region',
			summary: 'Get voice region',
			responseSchema: GetVoiceRegionResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Gets detailed information about a voice region including assigned servers, capacity, and server details. Requires VOICE_REGION_LIST permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.voiceService.getVoiceRegion(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/voice/regions/create',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_REGION_CREATE),
		Validator('json', CreateVoiceRegionRequest),
		OpenAPI({
			operationId: 'create_voice_region',
			summary: 'Create voice region',
			responseSchema: CreateVoiceRegionResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Creates a new voice server region. Defines geographic location and performance characteristics for voice routing. Creates audit log entry. Requires VOICE_REGION_CREATE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.createVoiceRegion(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/voice/regions/update',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_REGION_UPDATE),
		Validator('json', UpdateVoiceRegionRequest),
		OpenAPI({
			operationId: 'update_voice_region',
			summary: 'Update voice region',
			responseSchema: UpdateVoiceRegionResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Updates voice region settings such as latency thresholds or priority. Changes affect voice routing for new sessions. Creates audit log entry. Requires VOICE_REGION_UPDATE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.updateVoiceRegion(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/voice/regions/delete',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_REGION_DELETE),
		Validator('json', DeleteVoiceRegionRequest),
		OpenAPI({
			operationId: 'delete_voice_region',
			summary: 'Delete voice region',
			responseSchema: DeleteVoiceResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Deletes a voice region. Removes region from routing and reassigns active connections. Creates audit log entry. Requires VOICE_REGION_DELETE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.deleteVoiceRegion(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/voice/servers/list',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_SERVER_LIST),
		Validator('json', ListVoiceServersRequest),
		OpenAPI({
			operationId: 'list_voice_servers',
			summary: 'List voice servers',
			responseSchema: ListVoiceServersResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Lists all voice servers with connection counts and capacity. Shows server status, region assignment, and load information. Supports filtering and pagination. Requires VOICE_SERVER_LIST permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.voiceService.listVoiceServers(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/voice/servers/get',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_SERVER_LIST),
		Validator('json', GetVoiceServerRequest),
		OpenAPI({
			operationId: 'get_voice_server',
			summary: 'Get voice server',
			responseSchema: GetVoiceServerResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Gets detailed voice server information including active connections and configuration. Requires VOICE_SERVER_LIST permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.voiceService.getVoiceServer(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/voice/servers/create',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_SERVER_CREATE),
		Validator('json', CreateVoiceServerRequest),
		OpenAPI({
			operationId: 'create_voice_server',
			summary: 'Create voice server',
			responseSchema: CreateVoiceServerResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Creates and provisions a new voice server instance in a region. Configures capacity, codecs, and encryption. Creates audit log entry. Requires VOICE_SERVER_CREATE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.createVoiceServer(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/voice/servers/update',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_SERVER_UPDATE),
		Validator('json', UpdateVoiceServerRequest),
		OpenAPI({
			operationId: 'update_voice_server',
			summary: 'Update voice server',
			responseSchema: UpdateVoiceServerResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Updates voice server configuration including capacity, region assignment, and quality settings. Changes apply to new connections. Creates audit log entry. Requires VOICE_SERVER_UPDATE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.updateVoiceServer(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/voice/servers/delete',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.VOICE_SERVER_DELETE),
		Validator('json', DeleteVoiceServerRequest),
		OpenAPI({
			operationId: 'delete_voice_server',
			summary: 'Delete voice server',
			responseSchema: DeleteVoiceResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Decommissions and removes a voice server instance. Disconnects active sessions and migrates to other servers. Creates audit log entry. Requires VOICE_SERVER_DELETE permission.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.voiceService.deleteVoiceServer(ctx.req.valid('json'), adminUserId, auditLogReason),
			);
		},
	);
}
