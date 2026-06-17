// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../../BrandedTypes';
import type {GuildRow} from '../../database/types/GuildTypes';
import type {Guild} from '../../models/Guild';

export abstract class IGuildDataRepository {
	abstract findUnique(guildId: GuildID): Promise<Guild | null>;

	abstract listGuilds(guildIds: Array<GuildID>): Promise<Array<Guild>>;

	abstract listAllGuildsPaginated(limit: number, lastGuildId?: GuildID): Promise<Array<Guild>>;

	abstract listUserGuilds(userId: UserID): Promise<Array<Guild>>;

	abstract countUserGuilds(userId: UserID): Promise<number>;

	abstract listOwnedGuildIds(userId: UserID): Promise<Array<GuildID>>;

	abstract upsert(data: GuildRow, oldData?: GuildRow | null, previousOwnerId?: UserID): Promise<Guild>;

	abstract upsertPartial(
		guildId: GuildID,
		patch: Partial<GuildRow>,
		oldData?: GuildRow | null,
		previousOwnerId?: UserID,
	): Promise<Guild>;

	abstract delete(guildId: GuildID, ownerId?: UserID): Promise<void>;
}
