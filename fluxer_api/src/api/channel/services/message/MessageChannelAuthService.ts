// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildMemberResponse} from '@fluxer/schema/src/domains/guild/GuildMemberSchemas';
import type {GuildResponse} from '@fluxer/schema/src/domains/guild/GuildResponseSchemas';
import type {User} from '../../../models/User';
import {checkGuildVerificationWithResponse} from '../../../utils/GuildVerificationUtils';
import {BaseChannelAuthService, type ChannelAuthOptions} from '../BaseChannelAuthService';

export class MessageChannelAuthService extends BaseChannelAuthService {
	protected readonly options: ChannelAuthOptions = {
		errorOnMissingGuild: 'unknown_channel',
		validateNsfw: true,
	};

	async checkGuildVerification({
		user,
		guild,
		member,
	}: {
		user: User;
		guild: GuildResponse;
		member: GuildMemberResponse;
	}): Promise<void> {
		checkGuildVerificationWithResponse({user, guild, member});
	}
}
