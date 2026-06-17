// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IUserAccountRepository} from './IUserAccountRepository';
import type {IUserAuthRepository} from './IUserAuthRepository';
import type {IUserChannelRepository} from './IUserChannelRepository';
import type {IUserContentRepository} from './IUserContentRepository';
import type {IUserRelationshipRepository} from './IUserRelationshipRepository';
import type {IUserSettingsRepository} from './IUserSettingsRepository';

export interface IUserRepositoryAggregate
	extends IUserAccountRepository,
		IUserAuthRepository,
		IUserSettingsRepository,
		IUserRelationshipRepository,
		IUserChannelRepository,
		IUserContentRepository {}
