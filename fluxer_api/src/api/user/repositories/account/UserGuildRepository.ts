// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../../../BrandedTypes';
import {BatchBuilder, fetchMany} from '../../../database/CassandraQueryExecution';
import type {GuildMemberByUserIdRow} from '../../../database/types/GuildTypes';
import {GuildMembers, GuildMembersByUserId} from '../../../Tables';

const FETCH_GUILD_MEMBERS_BY_USER_CQL = GuildMembersByUserId.selectCql({
	where: GuildMembersByUserId.where.eq('user_id'),
});

export class UserGuildRepository {
	async getUserGuildIds(userId: UserID): Promise<Array<GuildID>> {
		const guilds = await fetchMany<GuildMemberByUserIdRow>(FETCH_GUILD_MEMBERS_BY_USER_CQL, {
			user_id: userId,
		});
		return guilds.map((g) => g.guild_id);
	}

	async removeFromAllGuilds(userId: UserID): Promise<void> {
		const guilds = await fetchMany<GuildMemberByUserIdRow>(FETCH_GUILD_MEMBERS_BY_USER_CQL, {
			user_id: userId,
		});
		const batch = new BatchBuilder();
		for (const guild of guilds) {
			batch.addPrepared(
				GuildMembers.deleteByPk({
					guild_id: guild.guild_id,
					user_id: userId,
				}),
			);
			batch.addPrepared(
				GuildMembersByUserId.deleteByPk({
					user_id: userId,
					guild_id: guild.guild_id,
				}),
			);
		}
		await batch.execute();
	}
}
