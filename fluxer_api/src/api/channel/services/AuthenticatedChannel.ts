// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildMemberResponse} from '@fluxer/schema/src/domains/guild/GuildMemberSchemas';
import type {GuildResponse} from '@fluxer/schema/src/domains/guild/GuildResponseSchemas';
import type {Channel} from '../../models/Channel';

export interface AuthenticatedChannel {
	channel: Channel;
	guild: GuildResponse | null;
	member: GuildMemberResponse | null;
	hasPermission: (permission: bigint) => Promise<boolean>;
	checkPermission: (permission: bigint) => Promise<void>;
}
