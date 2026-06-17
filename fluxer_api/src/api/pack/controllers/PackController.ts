// SPDX-License-Identifier: AGPL-3.0-or-later

import {PackIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {
	PackCreateRequest,
	PackDashboardResponse,
	PackSummaryResponse,
	PackTypeParam,
	PackUpdateRequest,
} from '@fluxer/schema/src/domains/pack/PackSchemas';
import {createGuildID} from '../../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';
import {mapPackToSummary} from '../PackModel';

export function PackController(app: HonoApp) {
	app.get(
		'/packs',
		RateLimitMiddleware(RateLimitConfigs.PACKS_LIST),
		LoginRequired,
		DefaultUserOnly,
		OpenAPI({
			operationId: 'list_user_packs',
			summary: 'List user packs',
			description:
				'Returns a dashboard view containing all emoji and sticker packs created by or owned by the authenticated user. This includes pack metadata such as name, description, type, and cover image.',
			responseSchema: PackDashboardResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			const response = await ctx.get('packService').listUserPacks(ctx.get('user').id);
			return ctx.json(response);
		},
	);
	app.post(
		'/packs/:pack_type',
		RateLimitMiddleware(RateLimitConfigs.PACKS_CREATE),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', PackTypeParam),
		Validator('json', PackCreateRequest),
		OpenAPI({
			operationId: 'create_pack',
			summary: 'Create pack',
			description:
				'Creates a new emoji or sticker pack owned by the authenticated user. The pack type is specified in the path parameter and can be either "emoji" or "sticker". Returns the newly created pack with its metadata.',
			responseSchema: PackSummaryResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			const user = ctx.get('user');
			const data = ctx.req.valid('json');
			const pack = await ctx.get('packService').createPack({
				user,
				type: ctx.req.valid('param').pack_type as 'emoji' | 'sticker',
				name: data.name,
				description: data.description ?? null,
			});
			return ctx.json(mapPackToSummary(pack));
		},
	);
	app.patch(
		'/packs/:pack_id',
		RateLimitMiddleware(RateLimitConfigs.PACKS_UPDATE),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', PackIdParam),
		Validator('json', PackUpdateRequest),
		OpenAPI({
			operationId: 'update_pack',
			summary: 'Update pack',
			description:
				'Updates the metadata for an existing pack owned by the authenticated user. Allowed modifications include name, description, and cover image. Returns the updated pack with all current metadata.',
			responseSchema: PackSummaryResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			const data = ctx.req.valid('json');
			const updated = await ctx.get('packService').updatePack({
				userId: ctx.get('user').id,
				packId: createGuildID(ctx.req.valid('param').pack_id),
				name: data.name,
				description: data.description,
			});
			return ctx.json(mapPackToSummary(updated));
		},
	);
	app.delete(
		'/packs/:pack_id',
		RateLimitMiddleware(RateLimitConfigs.PACKS_DELETE),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', PackIdParam),
		OpenAPI({
			operationId: 'delete_pack',
			summary: 'Delete pack',
			description:
				'Permanently deletes a pack owned by the authenticated user along with all emojis or stickers contained within it. This action cannot be undone and will remove all associated assets.',
			responseSchema: null,
			statusCode: 204,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			await ctx.get('packService').deletePack(ctx.get('user').id, createGuildID(ctx.req.valid('param').pack_id));
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/packs/:pack_id/install',
		RateLimitMiddleware(RateLimitConfigs.PACKS_INSTALL),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', PackIdParam),
		OpenAPI({
			operationId: 'install_pack',
			summary: 'Install pack',
			description:
				"Installs a pack to the authenticated user's collection, making its emojis or stickers available for use. The pack must be publicly accessible or owned by the user.",
			responseSchema: null,
			statusCode: 204,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			await ctx.get('packService').installPack(ctx.get('user').id, createGuildID(ctx.req.valid('param').pack_id));
			return ctx.body(null, 204);
		},
	);
	app.delete(
		'/packs/:pack_id/install',
		RateLimitMiddleware(RateLimitConfigs.PACKS_INSTALL),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', PackIdParam),
		OpenAPI({
			operationId: 'uninstall_pack',
			summary: 'Uninstall pack',
			description:
				"Uninstalls a pack from the authenticated user's collection, removing access to its emojis or stickers. This does not delete the pack itself, only removes it from the user's installed list.",
			responseSchema: null,
			statusCode: 204,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Packs'],
		}),
		async (ctx) => {
			await ctx.get('packService').uninstallPack(ctx.get('user').id, createGuildID(ctx.req.valid('param').pack_id));
			return ctx.body(null, 204);
		},
	);
}
