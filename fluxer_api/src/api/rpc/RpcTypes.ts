// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ChannelID, GuildID, UserID} from '../BrandedTypes';
import type {Channel} from '../models/Channel';
import type {FavoriteMeme} from '../models/FavoriteMeme';
import type {ReadState} from '../models/ReadState';
import type {Relationship} from '../models/Relationship';
import type {User} from '../models/User';
import type {UserGuildSettings} from '../models/UserGuildSettings';
import type {UserSettings} from '../models/UserSettings';
import type {WebAuthnCredential} from '../models/WebAuthnCredential';

export interface UserData {
	user: User;
	settings: UserSettings | null;
	guildSettings: Array<UserGuildSettings>;
	notes: Map<UserID, string>;
	readStates: Array<ReadState>;
	guildIds: Array<GuildID>;
	privateChannels: Array<Channel>;
	relationships: Array<Relationship>;
	favoriteMemes: Array<FavoriteMeme>;
	pinnedDMs: Array<ChannelID>;
	webAuthnCredentials: Array<WebAuthnCredential>;
}
