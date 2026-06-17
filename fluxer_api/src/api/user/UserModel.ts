// SPDX-License-Identifier: AGPL-3.0-or-later

import {MAX_GUILDS_PREMIUM} from '@fluxer/constants/src/LimitConstants';
import {Locales} from '@fluxer/constants/src/Locales';
import {StatusTypes} from '@fluxer/constants/src/StatusConstants';
import {DEFAULT_GUILD_FOLDER_ICON, GuildFolderIcons, ThemeTypes} from '@fluxer/constants/src/UserConstants';
import {ValidationErrorCodes} from '@fluxer/constants/src/ValidationErrorCodes';
import type {ValueOf} from '@fluxer/constants/src/ValueOf';
import {
	isValidSyncedPreferencesEncoding,
	SYNCED_PREFERENCES_MAX_ENCODED_LENGTH,
} from '@fluxer/schema/src/domains/user/SyncedPreferencesCodec';
import {CustomStatusPayload} from '@fluxer/schema/src/domains/user/UserRequestSchemas';
import {DateTimeType} from '@fluxer/schema/src/primitives/QueryValidators';
import {ColorType, createStringType, Int32Type, SnowflakeType} from '@fluxer/schema/src/primitives/SchemaPrimitives';
import {z} from 'zod';

const SettableStatusTypes = {
	ONLINE: StatusTypes.ONLINE,
	DND: StatusTypes.DND,
	IDLE: StatusTypes.IDLE,
	INVISIBLE: StatusTypes.INVISIBLE,
} as const;
const StatusTypeValues = Object.values(SettableStatusTypes) as Array<ValueOf<typeof SettableStatusTypes>>;
const ThemeTypeValues = Object.values(ThemeTypes) as Array<ValueOf<typeof ThemeTypes>>;
const LocaleValues = Object.values(Locales) as Array<ValueOf<typeof Locales>>;
const GuildFolderIconValues = Object.values(GuildFolderIcons) as Array<ValueOf<typeof GuildFolderIcons>>;
const StatusTypeSchema = z.enum(
	StatusTypeValues as [ValueOf<typeof SettableStatusTypes>, ...Array<ValueOf<typeof SettableStatusTypes>>],
);
const ThemeTypeSchema = z.enum(ThemeTypeValues as [ValueOf<typeof ThemeTypes>, ...Array<ValueOf<typeof ThemeTypes>>]);
const LocaleSchema = z.enum(LocaleValues as [ValueOf<typeof Locales>, ...Array<ValueOf<typeof Locales>>]);
const GuildFolderIconSchema = z.enum(
	GuildFolderIconValues as [ValueOf<typeof GuildFolderIcons>, ...Array<ValueOf<typeof GuildFolderIcons>>],
);
export const UserSettingsUpdateRequest = z
	.object({
		flags: z.number().int().describe('Bitfield of user settings flags'),
		status: StatusTypeSchema.describe('Current online status (online, idle, dnd, invisible)'),
		status_resets_at: DateTimeType.nullish().describe('When the status should reset'),
		status_resets_to: StatusTypeSchema.nullish().describe('Status to reset to after timer'),
		theme: ThemeTypeSchema.describe('UI theme preference (dark or light)'),
		guild_positions: z
			.array(SnowflakeType)
			.transform((ids) => [...new Set(ids)])
			.refine((ids) => ids.length <= MAX_GUILDS_PREMIUM, `Maximum ${MAX_GUILDS_PREMIUM} guilds allowed`)
			.describe('Ordered array of guild IDs for sidebar positioning'),
		locale: LocaleSchema.describe('User language/locale preference'),
		restricted_guilds: z
			.array(SnowflakeType)
			.transform((ids) => [...new Set(ids)])
			.refine((ids) => ids.length <= MAX_GUILDS_PREMIUM, `Maximum ${MAX_GUILDS_PREMIUM} guilds allowed`)
			.describe('Guild IDs where DMs from members are restricted'),
		bot_restricted_guilds: z
			.array(SnowflakeType)
			.transform((ids) => [...new Set(ids)])
			.refine((ids) => ids.length <= MAX_GUILDS_PREMIUM, `Maximum ${MAX_GUILDS_PREMIUM} guilds allowed`)
			.describe('Guild IDs where DMs from bots are restricted'),
		default_guilds_restricted: z.boolean().describe('Default DM restriction for new guilds'),
		bot_default_guilds_restricted: z.boolean().describe('Default bot DM restriction for new guilds'),
		inline_attachment_media: z.boolean().describe('Auto-display images and videos inline'),
		inline_embed_media: z.boolean().describe('Auto-display embedded media inline'),
		gif_auto_play: z.boolean().describe('Auto-play GIFs when visible'),
		render_embeds: z.boolean().describe('Show link embeds in messages'),
		render_reactions: z.boolean().describe('Show reactions on messages'),
		animate_emoji: z.boolean().describe('Animate custom emoji'),
		animate_stickers: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Sticker animation setting (0=never, 1=on hover, 2=always)'),
		render_spoilers: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Spoiler display setting (0=hidden, 1=on hover, 2=always)'),
		message_display_compact: z.boolean().describe('Use compact message display mode'),
		friend_source_flags: Int32Type.describe('Bitfield for friend request source permissions'),
		incoming_call_flags: Int32Type.describe('Bitfield for incoming call permissions'),
		group_dm_add_permission_flags: Int32Type.describe('Bitfield for group DM add permissions'),
		guild_folders: z
			.array(
				z.object({
					id: z.number().int().min(-1).describe('Unique folder identifier (-1 for uncategorized)'),
					name: createStringType(0, 100).nullish().describe('Folder display name'),
					color: ColorType.nullish().default(0x000000).describe('Folder color as integer'),
					flags: Int32Type.default(0).describe('Bitfield for guild folder display behaviour'),
					icon: GuildFolderIconSchema.default(DEFAULT_GUILD_FOLDER_ICON).describe('Selected icon for the guild folder'),
					guild_ids: z
						.array(SnowflakeType)
						.transform((ids) => [...new Set(ids)])
						.refine((ids) => ids.length <= MAX_GUILDS_PREMIUM, `Maximum ${MAX_GUILDS_PREMIUM} guilds allowed`)
						.describe('Guild IDs contained in this folder'),
				}),
			)
			.max(100)
			.describe('Array of guild folder configurations'),
		custom_status: CustomStatusPayload.nullish().describe('Custom status with text and emoji'),
		afk_timeout: z.number().int().min(60).max(600).describe('AFK timeout in seconds (60-600)'),
		time_format: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Time format preference (0=12h, 1=24h, 2=relative)'),
		developer_mode: z.boolean().describe('Enable developer mode features'),
		trusted_domains: z
			.array(z.string().min(1).max(253))
			.max(1000)
			.describe('Trusted external link domains. Use "*" to trust all domains.'),
		default_hide_muted_channels: z.boolean().describe('Hide muted channels by default in new guilds'),
		sensitive_content_friend_dm_filter: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Sensitive media filter level for DMs from friends (0=show, 1=blur, 2=block)'),
		sensitive_content_non_friend_dm_filter: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Sensitive media filter level for DMs from non-friends (0=show, 1=blur, 2=block)'),
		sensitive_content_guild_filter: z
			.union([z.literal(0), z.literal(1)])
			.describe('Sensitive media filter level for community channels (0=show, 1=blur)'),
		suppress_unprivileged_self_mentions: z
			.boolean()
			.describe('Suppress direct mentions and reply mentions from unprivileged users'),
		suppress_unprivileged_self_mentions_bypass_user_ids: z
			.array(SnowflakeType)
			.nullish()
			.describe('User IDs that bypass self-mention suppression'),
		staff_dm_access_user_ids: z.array(SnowflakeType).nullish().describe('User IDs with Staff DM Access enabled'),
		profile_privacy: z
			.union([z.literal(0), z.literal(1), z.literal(2)])
			.describe('Profile privacy level (0=all guild members, 1=small guilds only (<=200 members), 2=friends only)'),
		default_share_voice_activity: z
			.boolean()
			.describe('Default share_voice_activity applied to new friend relationships'),
		synced_preferences: z
			.string()
			.max(SYNCED_PREFERENCES_MAX_ENCODED_LENGTH)
			.refine((value) => isValidSyncedPreferencesEncoding(value), {
				message: ValidationErrorCodes.INVALID_FORMAT,
			})
			.nullish()
			.describe(
				'Account-wide client preferences as a base64-encoded protobuf snapshot. ' +
					'Replaces the entire stored snapshot; pass null to clear it.',
			),
	})
	.partial();
