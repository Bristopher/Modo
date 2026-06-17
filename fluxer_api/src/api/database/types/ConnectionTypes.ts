// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ConnectionType} from '@fluxer/constants/src/ConnectionConstants';
import type {UserID} from '../../BrandedTypes';

type Nullish<T> = T | null;

export interface UserConnectionRow {
	user_id: UserID;
	connection_id: string;
	connection_type: ConnectionType;
	identifier: string;
	name: string;
	verified: boolean;
	visibility_flags: number;
	sort_order: number;
	verification_token: string;
	verified_at: Nullish<Date>;
	last_verified_at: Nullish<Date>;
	created_at: Date;
	version: number;
}

export const USER_CONNECTION_COLUMNS = [
	'user_id',
	'connection_id',
	'connection_type',
	'identifier',
	'name',
	'verified',
	'visibility_flags',
	'sort_order',
	'verification_token',
	'verified_at',
	'last_verified_at',
	'created_at',
	'version',
] as const satisfies ReadonlyArray<keyof UserConnectionRow>;
