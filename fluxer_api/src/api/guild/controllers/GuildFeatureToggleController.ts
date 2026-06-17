// SPDX-License-Identifier: AGPL-3.0-or-later

import {GuildFeatures} from '@fluxer/constants/src/GuildConstants';
import {
	DisabledToggleRequest,
	EnabledToggleRequest,
	GuildIdParam,
} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import type {GuildUpdateRequest} from '@fluxer/schema/src/domains/guild/GuildRequestSchemas';
import {GuildResponse} from '@fluxer/schema/src/domains/guild/GuildResponseSchemas';
import {createGuildID} from '../../BrandedTypes';
import {LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

type ToggleMode = 'enabled' | 'disabled';
type GuildFeatureToggleConfig = {
	path: string;
	operationId: string;
	summary: string;
	description: string;
	feature: string;
	mode: ToggleMode;
};

const TOGGLES: ReadonlyArray<GuildFeatureToggleConfig> = [
	{
		path: '/guilds/:guild_id/text-channel-flexible-names',
		operationId: 'toggle_text_channel_flexible_names',
		summary: 'Toggle text channel flexible names',
		description: 'Requires manage_guild permission. Allows or disables flexible naming for text channels.',
		feature: GuildFeatures.TEXT_CHANNEL_FLEXIBLE_NAMES,
		mode: 'enabled',
	},
	{
		path: '/guilds/:guild_id/detached-banner',
		operationId: 'toggle_detached_banner',
		summary: 'Toggle detached banner',
		description: 'Requires manage_guild permission. Enables or disables independent banner display configuration.',
		feature: GuildFeatures.DETACHED_BANNER,
		mode: 'enabled',
	},
	{
		path: '/guilds/:guild_id/invites-disabled',
		operationId: 'toggle_invites_disabled',
		summary: 'Toggle invites disabled',
		description: 'Requires manage_guild permission. Pauses or resumes invite-link joins for this guild.',
		feature: GuildFeatures.INVITES_DISABLED,
		mode: 'disabled',
	},
	{
		path: '/guilds/:guild_id/clone-emoji-disabled',
		operationId: 'toggle_clone_emoji_disabled',
		summary: 'Toggle emoji cloning disabled',
		description:
			"Requires manage_guild permission. When disabled, members of other guilds cannot use the in-app one-click clone shortcut for this guild's emojis. Note that this does not prevent users from saving and re-uploading the image manually.",
		feature: GuildFeatures.CLONE_EMOJI_DISABLED,
		mode: 'disabled',
	},
	{
		path: '/guilds/:guild_id/hide-owner-crown',
		operationId: 'toggle_hide_owner_crown',
		summary: 'Toggle hide community owner crown',
		description:
			'Requires manage_guild permission. When enabled, the community owner crown icon is hidden across the UI for this guild.',
		feature: GuildFeatures.HIDE_OWNER_CROWN,
		mode: 'enabled',
	},
	{
		path: '/guilds/:guild_id/clone-sticker-disabled',
		operationId: 'toggle_clone_sticker_disabled',
		summary: 'Toggle sticker cloning disabled',
		description:
			"Requires manage_guild permission. When disabled, members of other guilds cannot use the in-app one-click clone shortcut for this guild's stickers. Note that this does not prevent users from saving and re-uploading the image manually.",
		feature: GuildFeatures.CLONE_STICKER_DISABLED,
		mode: 'disabled',
	},
];

function registerGuildFeatureToggle(app: HonoApp, config: GuildFeatureToggleConfig) {
	const bodySchema = config.mode === 'enabled' ? EnabledToggleRequest : DisabledToggleRequest;
	app.patch(
		config.path,
		RateLimitMiddleware(RateLimitConfigs.GUILD_UPDATE),
		LoginRequired,
		Validator('param', GuildIdParam),
		Validator('json', bodySchema),
		OpenAPI({
			operationId: config.operationId,
			summary: config.summary,
			description: config.description,
			responseSchema: GuildResponse,
			statusCode: 200,
			security: ['botToken', 'bearerToken', 'sessionToken'],
			tags: ['Guilds'],
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const guildId = createGuildID(ctx.req.valid('param').guild_id);
			const body = ctx.req.valid('json');
			const present =
				config.mode === 'enabled' ? (body as EnabledToggleRequest).enabled : (body as DisabledToggleRequest).disabled;
			const requestCache = ctx.get('requestCache');
			const auditLogReason = ctx.get('auditLogReason') ?? null;
			const guildService = ctx.get('guildService');
			const currentFeatures = await guildService.getGuildFeaturesForToggle(guildId);
			const features = new Set(currentFeatures);
			if (present) {
				features.add(config.feature);
			} else {
				features.delete(config.feature);
			}
			const data: GuildUpdateRequest = {features: Array.from(features)};
			return ctx.json(await guildService.updateGuild({userId, guildId, data, requestCache}, auditLogReason));
		},
	);
}

export function GuildFeatureToggleController(app: HonoApp) {
	for (const config of TOGGLES) {
		registerGuildFeatureToggle(app, config);
	}
}
