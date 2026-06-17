// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../../BrandedTypes';
import type {ExactRow} from '../../database/types/DatabaseRowTypes';
import type {UserGuildSettingsRow, UserSettingsRow} from '../../database/types/UserTypes';
import type {UserGuildSettings} from '../../models/UserGuildSettings';
import type {UserSettings} from '../../models/UserSettings';

export interface IUserSettingsRepository {
	findSettings(userId: UserID): Promise<UserSettings | null>;
	upsertSettings(settings: ExactRow<UserSettingsRow>): Promise<UserSettings>;
	deleteUserSettings(userId: UserID): Promise<void>;
	findGuildSettings(userId: UserID, guildId: GuildID | null): Promise<UserGuildSettings | null>;
	findAllGuildSettings(userId: UserID): Promise<Array<UserGuildSettings>>;
	upsertGuildSettings(settings: ExactRow<UserGuildSettingsRow>): Promise<UserGuildSettings>;
	deleteGuildSettings(userId: UserID, guildId: GuildID): Promise<void>;
	deleteAllUserGuildSettings(userId: UserID): Promise<void>;
}
