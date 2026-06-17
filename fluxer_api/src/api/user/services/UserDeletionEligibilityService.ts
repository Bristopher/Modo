// SPDX-License-Identifier: AGPL-3.0-or-later

import {UserFlags} from '@fluxer/constants/src/UserConstants';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import {ms, seconds} from 'itty-time';
import type {UserID} from '../../BrandedTypes';
import {Config} from '../../Config';
import type {User} from '../../models/User';

export class UserDeletionEligibilityService {
	private readonly INACTIVITY_WARNING_TTL_DAYS = 30;
	private readonly INACTIVITY_WARNING_PREFIX = 'inactivity_warning_sent';

	constructor(private kvClient: IKVProvider) {}

	async isEligibleForInactivityDeletion(user: User): Promise<boolean> {
		if (user.isBot) {
			return false;
		}
		if (user.isSystem) {
			return false;
		}
		if (this.isAppStoreReviewer(user)) {
			return false;
		}
		if (user.pendingDeletionAt !== null) {
			return false;
		}
		if (user.lastActiveAt === null) {
			return false;
		}
		const inactivityThresholdMs = this.getInactivityThresholdMs();
		const timeSinceLastActiveMs = Date.now() - user.lastActiveAt.getTime();
		if (timeSinceLastActiveMs < inactivityThresholdMs) {
			return false;
		}
		return true;
	}

	async isEligibleForWarningEmail(user: User): Promise<boolean> {
		const isEligibleForDeletion = await this.isEligibleForInactivityDeletion(user);
		if (!isEligibleForDeletion) {
			return false;
		}
		const alreadySentWarning = await this.hasWarningSent(user.id);
		if (alreadySentWarning) {
			return false;
		}
		return true;
	}

	async markWarningSent(userId: UserID): Promise<void> {
		const key = this.getWarningKey(userId);
		const ttlSeconds = seconds(`${this.INACTIVITY_WARNING_TTL_DAYS + 5} days`);
		const timestamp = Date.now().toString();
		await this.kvClient.setex(key, ttlSeconds, timestamp);
	}

	async hasWarningSent(userId: UserID): Promise<boolean> {
		const key = this.getWarningKey(userId);
		const exists = await this.kvClient.exists(key);
		return exists === 1;
	}

	async getWarningSentTimestamp(userId: UserID): Promise<number | null> {
		const key = this.getWarningKey(userId);
		const value = await this.kvClient.get(key);
		if (!value) {
			return null;
		}
		const timestamp = parseInt(value, 10);
		return Number.isNaN(timestamp) ? null : timestamp;
	}

	async hasWarningGracePeriodExpired(userId: UserID): Promise<boolean> {
		const timestamp = await this.getWarningSentTimestamp(userId);
		if (timestamp === null) {
			return false;
		}
		const timeSinceWarningMs = Date.now() - timestamp;
		const gracePeriodMs = this.INACTIVITY_WARNING_TTL_DAYS * ms('1 day');
		return timeSinceWarningMs >= gracePeriodMs;
	}

	private getInactivityThresholdMs(): number {
		const thresholdDays = Config.inactivityDeletionThresholdDays ?? 365 * 2;
		return thresholdDays * ms('1 day');
	}

	private getWarningKey(userId: UserID): string {
		return `${this.INACTIVITY_WARNING_PREFIX}:${userId}`;
	}

	private isAppStoreReviewer(user: User): boolean {
		return (user.flags & UserFlags.APP_STORE_REVIEWER) !== 0n;
	}
}
