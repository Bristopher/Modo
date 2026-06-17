// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../../BrandedTypes';
import type {AdminApiKey} from '../../models/AdminApiKey';

export interface CreateAdminApiKeyData {
	name: string;
	expiresAt: Date | null;
	acls: Set<string>;
}

export interface IAdminApiKeyRepository {
	create(data: CreateAdminApiKeyData, createdBy: UserID, keyId: bigint, rawKey: string): Promise<AdminApiKey>;
	findById(keyId: bigint): Promise<AdminApiKey | null>;
	listByCreator(createdBy: UserID): Promise<Array<AdminApiKey>>;
	updateLastUsed(keyId: bigint, expiresAt: Date | null): Promise<void>;
	revoke(keyId: bigint, createdBy: UserID): Promise<void>;
}
