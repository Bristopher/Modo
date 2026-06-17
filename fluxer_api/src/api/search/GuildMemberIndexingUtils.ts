// SPDX-License-Identifier: AGPL-3.0-or-later

import {GUILD_MEMBERS_REINDEX_AFTER_TIMESTAMP} from '@fluxer/constants/src/GuildConstants';

export function guildMembersNeedReindexing(membersIndexedAt: Date | null | undefined): boolean {
	if (!membersIndexedAt) {
		return true;
	}
	const indexedAtSeconds = Math.floor(membersIndexedAt.getTime() / 1000);
	return indexedAtSeconds < GUILD_MEMBERS_REINDEX_AFTER_TIMESTAMP;
}
