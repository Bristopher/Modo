// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IGuildContentRepository} from './IGuildContentRepository';
import type {IGuildDataRepository} from './IGuildDataRepository';
import type {IGuildMemberRepository} from './IGuildMemberRepository';
import type {IGuildModerationRepository} from './IGuildModerationRepository';
import type {IGuildRoleRepository} from './IGuildRoleRepository';

export interface IGuildRepositoryAggregate
	extends IGuildDataRepository,
		IGuildMemberRepository,
		IGuildRoleRepository,
		IGuildModerationRepository,
		IGuildContentRepository {}
