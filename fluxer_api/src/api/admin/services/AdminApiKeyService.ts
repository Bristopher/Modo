// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomInt} from 'node:crypto';
import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {AdminApiKeyNotFoundError} from '@fluxer/errors/src/domains/admin/AdminApiKeyNotFoundError';
import {MissingACLError} from '@fluxer/errors/src/domains/core/MissingACLError';
import type {CreateAdminApiKeyRequest} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {ms} from 'itty-time';
import type {UserID} from '../../BrandedTypes';
import type {ISnowflakeService} from '../../infrastructure/ISnowflakeService';
import {verifyPassword} from '../../utils/PasswordUtils';
import type {IAdminApiKeyRepository} from '../repositories/IAdminApiKeyRepository';

const ADMIN_KEY_PREFIX = 'fa_';
const RANDOM_KEY_LENGTH = 32;
const CHARSET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';

interface CreateApiKeyResult {
	key: string;
	apiKey: {
		keyId: string;
		name: string;
		createdAt: Date;
		expiresAt: Date | null;
		acls: Set<string>;
	};
}

export class AdminApiKeyService {
	constructor(
		private readonly adminApiKeyRepository: IAdminApiKeyRepository,
		private readonly snowflakeService: ISnowflakeService,
	) {}

	private generateRawKey(keyId: bigint): string {
		const randomChars = Array.from({length: RANDOM_KEY_LENGTH}, () => CHARSET[randomInt(CHARSET.length)]).join('');
		return `${ADMIN_KEY_PREFIX}${keyId.toString()}_${randomChars}`;
	}

	private extractKeyId(rawKey: string): bigint | null {
		if (!rawKey.startsWith(ADMIN_KEY_PREFIX)) {
			return null;
		}
		const remainder = rawKey.slice(ADMIN_KEY_PREFIX.length);
		const underscoreIdx = remainder.indexOf('_');
		if (underscoreIdx <= 0) {
			return null;
		}
		const keyIdStr = remainder.slice(0, underscoreIdx);
		if (!/^\d+$/.test(keyIdStr)) {
			return null;
		}
		try {
			return BigInt(keyIdStr);
		} catch {
			return null;
		}
	}

	async createApiKey(
		request: CreateAdminApiKeyRequest,
		createdBy: UserID,
		creatorAcls?: Set<string>,
	): Promise<CreateApiKeyResult> {
		const keyId = await this.snowflakeService.generate();
		const rawKey = this.generateRawKey(keyId);
		const expiresAt = request.expires_in_days ? new Date(Date.now() + request.expires_in_days * ms('1 day')) : null;
		if (creatorAcls) {
			const invalidACLs = request.acls.filter((acl) => !creatorAcls.has(acl) && !creatorAcls.has(AdminACLs.WILDCARD));
			if (invalidACLs.length > 0) {
				throw new MissingACLError(invalidACLs[0]);
			}
		}
		const aclsSet = new Set(request.acls);
		const apiKey = await this.adminApiKeyRepository.create(
			{
				name: request.name,
				expiresAt,
				acls: aclsSet,
			},
			createdBy,
			keyId,
			rawKey,
		);
		return {
			key: rawKey,
			apiKey: {
				keyId: apiKey.keyId.toString(),
				name: apiKey.name,
				createdAt: apiKey.createdAt,
				expiresAt: apiKey.expiresAt,
				acls: apiKey.acls,
			},
		};
	}

	async validateApiKey(rawKey: string): Promise<{
		keyId: bigint;
		createdById: UserID;
		acls: Set<string> | null;
	} | null> {
		const keyId = this.extractKeyId(rawKey);
		if (keyId === null) return null;
		const apiKey = await this.adminApiKeyRepository.findById(keyId);
		if (!apiKey) {
			return null;
		}
		if (apiKey.isExpired()) {
			return null;
		}
		const valid = await verifyPassword({password: rawKey, passwordHash: apiKey.keyHash});
		if (!valid) {
			return null;
		}
		await this.adminApiKeyRepository.updateLastUsed(apiKey.keyId, apiKey.expiresAt);
		return {
			keyId: apiKey.keyId,
			createdById: apiKey.createdById,
			acls: apiKey.acls,
		};
	}

	async listKeys(createdBy: UserID): Promise<
		Array<{
			keyId: string;
			name: string;
			createdAt: Date;
			lastUsedAt: Date | null;
			expiresAt: Date | null;
			createdById: UserID;
			acls: Set<string>;
		}>
	> {
		const apiKeys = await this.adminApiKeyRepository.listByCreator(createdBy);
		return apiKeys.map((key) => ({
			keyId: key.keyId.toString(),
			name: key.name,
			createdAt: key.createdAt,
			lastUsedAt: key.lastUsedAt,
			expiresAt: key.expiresAt,
			createdById: key.createdById,
			acls: key.acls ?? new Set(),
		}));
	}

	async revokeKey(keyId: string, createdBy: UserID): Promise<void> {
		if (!/^\d+$/.test(keyId)) {
			throw new AdminApiKeyNotFoundError();
		}
		const keyIdBigInt = BigInt(keyId);
		const apiKey = await this.adminApiKeyRepository.findById(keyIdBigInt);
		if (!apiKey) {
			throw new AdminApiKeyNotFoundError();
		}
		if (apiKey.createdById !== createdBy) {
			throw new AdminApiKeyNotFoundError();
		}
		await this.adminApiKeyRepository.revoke(keyIdBigInt, createdBy);
	}
}
