// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildAdminResponse, ListUserGuildsResponse} from '@fluxer/schema/src/domains/admin/AdminGuildSchemas';
import {mapGuildFeatures} from '../../guild/GuildFeatureUtils';
import type {Guild} from '../../models/Guild';
import type {User} from '../../models/User';

function formatOwnerFields(ownerUser: User | null): {
	owner_username: string | null;
	owner_global_name: string | null;
	owner_discriminator: string | null;
} {
	if (!ownerUser) {
		return {owner_username: null, owner_global_name: null, owner_discriminator: null};
	}
	return {
		owner_username: ownerUser.username,
		owner_global_name: ownerUser.globalName ?? null,
		owner_discriminator: String(ownerUser.discriminator).padStart(4, '0'),
	};
}

export function mapGuildToAdminResponse(guild: Guild, ownerUser?: User | null): GuildAdminResponse {
	return {
		id: guild.id.toString(),
		name: guild.name,
		features: mapGuildFeatures(guild.features),
		owner_id: guild.ownerId.toString(),
		...formatOwnerFields(ownerUser ?? null),
		icon: guild.iconHash,
		banner: guild.bannerHash,
		member_count: guild.memberCount,
		nsfw_level: guild.nsfwLevel,
	};
}

export function mapGuildsToAdminResponse(guilds: Array<Guild>, ownerMap?: Map<string, User>): ListUserGuildsResponse {
	return {
		guilds: guilds.map((guild) => {
			const ownerUser = ownerMap?.get(guild.ownerId.toString()) ?? null;
			return {
				id: guild.id.toString(),
				name: guild.name,
				features: mapGuildFeatures(guild.features),
				owner_id: guild.ownerId.toString(),
				...formatOwnerFields(ownerUser),
				icon: guild.iconHash,
				banner: guild.bannerHash,
				member_count: guild.memberCount,
				nsfw_level: guild.nsfwLevel,
			};
		}),
	};
}
