// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	BanGuildMemberRequest,
	ClearGuildFieldsRequest,
	DeleteGuildRequest,
	ForceAddUserToGuildRequest,
	KickGuildMemberRequest,
	ListGuildAuditLogsRequest,
	ListGuildAuditLogsResponse,
	ListGuildMembersRequest,
	LookupGuildRequest,
	ReloadGuildRequest,
	ShutdownGuildRequest,
	TransferGuildOwnershipRequest,
	UpdateGuildFeaturesRequest,
	UpdateGuildNameRequest,
	UpdateGuildSettingsRequest,
	UpdateGuildVanityRequest,
} from '@fluxer/schema/src/domains/admin/AdminGuildSchemas';
import {
	GuildUpdateResponse,
	ListGuildEmojisResponse,
	ListGuildMembersResponse,
	ListGuildStickersResponse,
	LookupGuildResponse,
	SuccessResponse,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {GuildIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {createGuildID} from '../../BrandedTypes';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import {AdminRateLimitConfigs} from '../../rate_limit_configs/AdminRateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function GuildAdminController(app: HonoApp) {
	app.post(
		'/admin/guilds/lookup',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_LOOKUP),
		Validator('json', LookupGuildRequest),
		OpenAPI({
			operationId: 'lookup_guild',
			summary: 'Look up guild',
			description:
				'Retrieves complete guild details including metadata, settings, and statistics. Look up by guild ID or vanity slug. Requires GUILD_LOOKUP permission.',
			responseSchema: LookupGuildResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.guildServiceAggregate.lookupService.lookupGuild(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/guilds/list-members',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_LIST_MEMBERS),
		Validator('json', ListGuildMembersRequest),
		OpenAPI({
			operationId: 'admin_list_guild_members',
			summary: 'List guild members',
			description:
				'Lists all guild members with pagination. Returns member IDs, join dates, and roles. Requires GUILD_LIST_MEMBERS permission.',
			responseSchema: ListGuildMembersResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.guildServiceAggregate.lookupService.listGuildMembers(ctx.req.valid('json')));
		},
	);
	app.get(
		'/admin/guilds/:guild_id/emojis',
		RateLimitMiddleware(AdminRateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.ASSET_PURGE),
		Validator('param', GuildIdParam),
		OpenAPI({
			operationId: 'admin_list_guild_emojis',
			summary: 'List guild emojis',
			description:
				'Lists all custom emojis in a guild. Returns ID, name, and creation date. Used for asset inventory and purge operations. Requires ASSET_PURGE permission.',
			responseSchema: ListGuildEmojisResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const guildId = createGuildID(ctx.req.valid('param').guild_id);
			return ctx.json(await adminService.guildServiceAggregate.lookupService.listGuildEmojis(guildId));
		},
	);
	app.get(
		'/admin/guilds/:guild_id/stickers',
		RateLimitMiddleware(AdminRateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.ASSET_PURGE),
		Validator('param', GuildIdParam),
		OpenAPI({
			operationId: 'admin_list_guild_stickers',
			summary: 'List guild stickers',
			description:
				'Lists all stickers in a guild. Returns ID, name, and asset information. Used for asset inventory and purge operations. Requires ASSET_PURGE permission.',
			responseSchema: ListGuildStickersResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const guildId = createGuildID(ctx.req.valid('param').guild_id);
			return ctx.json(await adminService.guildServiceAggregate.lookupService.listGuildStickers(guildId));
		},
	);
	app.post(
		'/admin/guilds/audit-logs',
		RateLimitMiddleware(AdminRateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.GUILD_AUDIT_LOG_VIEW),
		Validator('json', ListGuildAuditLogsRequest),
		OpenAPI({
			operationId: 'list_guild_audit_logs_admin',
			summary: 'List guild audit logs',
			description:
				'Returns in-app guild audit log entries for a guild without requiring VIEW_AUDIT_LOG membership permission. Supports pagination via before/after log IDs and filtering by user_id or action_type. Requires GUILD_AUDIT_LOG_VIEW permission.',
			responseSchema: ListGuildAuditLogsResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.guildServiceAggregate.listGuildAuditLogs(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/guilds/clear-fields',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_UPDATE_SETTINGS),
		Validator('json', ClearGuildFieldsRequest),
		OpenAPI({
			operationId: 'clear_guild_fields',
			summary: 'Clear guild fields',
			description:
				'Clears specified optional guild fields such as icon, banner, or description. Logged to audit log. Requires GUILD_UPDATE_SETTINGS permission.',
			responseSchema: null,
			statusCode: 204,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.guildServiceAggregate.updateService.clearGuildFields(
				ctx.req.valid('json'),
				adminUserId,
				auditLogReason,
			);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/guilds/update-features',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_UPDATE_FEATURES),
		Validator('json', UpdateGuildFeaturesRequest),
		OpenAPI({
			operationId: 'update_guild_features',
			summary: 'Update guild features',
			description:
				'Enables or disables guild feature flags. Modifies verification levels and community settings. Changes are logged to audit log. Requires GUILD_UPDATE_FEATURES permission.',
			responseSchema: GuildUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			const guildId = createGuildID(body.guild_id);
			return ctx.json(
				await adminService.guildServiceAggregate.updateService.updateGuildFeatures({
					guildId,
					addFeatures: body.add_features,
					removeFeatures: body.remove_features,
					adminUserId,
					auditLogReason,
				}),
			);
		},
	);
	app.post(
		'/admin/guilds/update-name',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_UPDATE_NAME),
		Validator('json', UpdateGuildNameRequest),
		OpenAPI({
			operationId: 'update_guild_name',
			summary: 'Update guild name',
			description:
				'Changes a guild name. Used for removing inappropriate names or correcting display issues. Logged to audit log. Requires GUILD_UPDATE_NAME permission.',
			responseSchema: GuildUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.guildServiceAggregate.updateService.updateGuildName(
					ctx.req.valid('json'),
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/update-settings',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_UPDATE_SETTINGS),
		Validator('json', UpdateGuildSettingsRequest),
		OpenAPI({
			operationId: 'update_guild_settings',
			summary: 'Update guild settings',
			description:
				'Modifies guild configuration including description, region, language and other settings. Logged to audit log. Requires GUILD_UPDATE_SETTINGS permission.',
			responseSchema: GuildUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.guildServiceAggregate.updateService.updateGuildSettings(
					ctx.req.valid('json'),
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/transfer-ownership',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_TRANSFER_OWNERSHIP),
		Validator('json', TransferGuildOwnershipRequest),
		OpenAPI({
			operationId: 'admin_transfer_guild_ownership',
			summary: 'Transfer guild ownership',
			description:
				'Transfers guild ownership to another user. Used when owner is inactive or for administrative recovery. Logged to audit log. Requires GUILD_TRANSFER_OWNERSHIP permission.',
			responseSchema: GuildUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.guildServiceAggregate.updateService.transferGuildOwnership(
					ctx.req.valid('json'),
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/update-vanity',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_UPDATE_VANITY),
		Validator('json', UpdateGuildVanityRequest),
		OpenAPI({
			operationId: 'update_guild_vanity',
			summary: 'Update guild vanity',
			description:
				'Updates a guild vanity URL slug. Sets custom short URL and prevents duplicate slugs. Logged to audit log. Requires GUILD_UPDATE_VANITY permission.',
			responseSchema: GuildUpdateResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			return ctx.json(
				await adminService.guildServiceAggregate.vanityService.updateGuildVanity(
					ctx.req.valid('json'),
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/force-add-user',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_FORCE_ADD_MEMBER),
		Validator('json', ForceAddUserToGuildRequest),
		OpenAPI({
			operationId: 'force_add_user_to_guild',
			summary: 'Force add user to guild',
			description:
				'Forcefully adds a user to a guild. Bypasses normal invite flow for administrative account recovery. Logged to audit log. Requires GUILD_FORCE_ADD_MEMBER permission.',
			responseSchema: SuccessResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const requestCache = ctx.get('requestCache');
			return ctx.json(
				await adminService.guildServiceAggregate.membershipService.forceAddUserToGuild({
					data: ctx.req.valid('json'),
					requestCache,
					adminUserId,
					auditLogReason,
				}),
			);
		},
	);
	app.post(
		'/admin/guilds/ban-member',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_BAN_MEMBER),
		Validator('json', BanGuildMemberRequest),
		OpenAPI({
			operationId: 'admin_ban_guild_member',
			summary: 'Ban guild member',
			description:
				'Permanently bans a user from a guild. Prevents user from joining. Logged to audit log. Requires GUILD_BAN_MEMBER permission.',
			responseSchema: null,
			statusCode: 204,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.guildServiceAggregate.membershipService.banMember(
				ctx.req.valid('json'),
				adminUserId,
				auditLogReason,
			);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/guilds/kick-member',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_KICK_MEMBER),
		Validator('json', KickGuildMemberRequest),
		OpenAPI({
			operationId: 'kick_guild_member',
			summary: 'Kick guild member',
			description:
				'Temporarily removes a user from a guild. User can rejoin. Logged to audit log. Requires GUILD_KICK_MEMBER permission.',
			responseSchema: null,
			statusCode: 204,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.guildServiceAggregate.membershipService.kickMember(
				ctx.req.valid('json'),
				adminUserId,
				auditLogReason,
			);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/guilds/reload',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_RELOAD),
		Validator('json', ReloadGuildRequest),
		OpenAPI({
			operationId: 'reload_guild',
			summary: 'Reload guild',
			description:
				'Reloads a single guild state from database. Used to recover from corruption or sync issues. Logged to audit log. Requires GUILD_RELOAD permission.',
			responseSchema: SuccessResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			return ctx.json(
				await adminService.guildServiceAggregate.managementService.reloadGuild(
					body.guild_id,
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/shutdown',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_SHUTDOWN),
		Validator('json', ShutdownGuildRequest),
		OpenAPI({
			operationId: 'shutdown_guild',
			summary: 'Shutdown guild',
			description:
				'Shuts down and unloads a guild from the gateway. Guild data remains in database. Used for emergency resource cleanup. Logged to audit log. Requires GUILD_SHUTDOWN permission.',
			responseSchema: SuccessResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			return ctx.json(
				await adminService.guildServiceAggregate.managementService.shutdownGuild(
					body.guild_id,
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
	app.post(
		'/admin/guilds/delete',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_GUILD_MODIFY),
		requireAdminACL(AdminACLs.GUILD_DELETE),
		Validator('json', DeleteGuildRequest),
		OpenAPI({
			operationId: 'admin_delete_guild',
			summary: 'Delete guild',
			description:
				'Permanently deletes a guild. Deletes all channels, messages, and settings. Irreversible operation. Logged to audit log. Requires GUILD_DELETE permission.',
			responseSchema: SuccessResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			return ctx.json(
				await adminService.guildServiceAggregate.managementService.deleteGuild(
					body.guild_id,
					adminUserId,
					auditLogReason,
				),
			);
		},
	);
}
