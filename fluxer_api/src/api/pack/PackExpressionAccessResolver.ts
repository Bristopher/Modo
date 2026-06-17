// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../BrandedTypes';
import type {PackType} from './PackRepository';

export type PackExpressionAccessResolution = 'accessible' | 'not-accessible' | 'not-pack';

export interface PackExpressionAccessResolver {
	resolve(packId: GuildID): Promise<PackExpressionAccessResolution>;
}

export interface PackExpressionAccessResolverParams {
	userId: UserID | null;
	type: PackType;
}
