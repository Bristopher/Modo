// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';

export interface ISessionTerminator {
	terminateAllUserSessions(userId: UserID): Promise<void>;
}
