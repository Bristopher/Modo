// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import type {UserID} from '../BrandedTypes';
import {Logger} from '../Logger';

interface QueuedBulkMessageDeletion {
	userId: bigint;
	scheduledAt: number;
}

const QUEUE_KEY = 'bulk_message_deletion_queue';
const SECONDARY_KEY_PREFIX = 'bulk_message_deletion_queue:';

export class KVBulkMessageDeletionQueueService {
	constructor(private readonly kvClient: IKVProvider) {}

	private getSecondaryKey(userId: UserID): string {
		return `${SECONDARY_KEY_PREFIX}${userId}`;
	}

	private serializeQueueItem(item: QueuedBulkMessageDeletion): string {
		return `${item.userId}|${item.scheduledAt}`;
	}

	private deserializeQueueItem(value: string): QueuedBulkMessageDeletion {
		const [userIdStr, scheduledAtStr] = value.split('|');
		return {
			userId: BigInt(userIdStr),
			scheduledAt: Number.parseInt(scheduledAtStr, 10),
		};
	}

	async scheduleDeletion(userId: UserID, scheduledAt: Date): Promise<void> {
		try {
			const entry: QueuedBulkMessageDeletion = {
				userId,
				scheduledAt: scheduledAt.getTime(),
			};
			const value = this.serializeQueueItem(entry);
			const secondaryKey = this.getSecondaryKey(userId);
			await this.kvClient.scheduleBulkDeletion(QUEUE_KEY, secondaryKey, entry.scheduledAt, value);
			Logger.debug({userId: userId.toString(), scheduledAt}, 'Scheduled bulk message deletion');
		} catch (error) {
			Logger.error({error, userId: userId.toString()}, 'Failed to schedule bulk message deletion');
			throw error;
		}
	}

	async removeFromQueue(userId: UserID): Promise<void> {
		try {
			const secondaryKey = this.getSecondaryKey(userId);
			const removed = await this.kvClient.removeBulkDeletion(QUEUE_KEY, secondaryKey);
			if (!removed) {
				Logger.debug({userId: userId.toString()}, 'User not in bulk message deletion queue');
				return;
			}
			Logger.debug({userId: userId.toString()}, 'Removed bulk message deletion from queue');
		} catch (error) {
			Logger.error({error, userId: userId.toString()}, 'Failed to remove bulk message deletion from queue');
			throw error;
		}
	}

	async getReadyDeletions(nowMs: number, limit: number): Promise<Array<QueuedBulkMessageDeletion>> {
		try {
			const results = await this.kvClient.zrangebyscore(QUEUE_KEY, '-inf', nowMs, 'LIMIT', 0, limit);
			const deletions: Array<QueuedBulkMessageDeletion> = [];
			for (const result of results) {
				try {
					const deletion = this.deserializeQueueItem(result);
					deletions.push(deletion);
				} catch (error) {
					Logger.error({error, result}, 'Failed to parse queued bulk message deletion entry');
				}
			}
			return deletions;
		} catch (error) {
			Logger.error({error, nowMs, limit}, 'Failed to fetch ready bulk message deletions');
			throw error;
		}
	}

	async getQueueSize(): Promise<number> {
		try {
			return await this.kvClient.zcard(QUEUE_KEY);
		} catch (error) {
			Logger.error({error}, 'Failed to get bulk message deletion queue size');
			throw error;
		}
	}
}
