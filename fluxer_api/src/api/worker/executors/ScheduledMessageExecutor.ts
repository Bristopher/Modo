// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageID, UserID} from '../../BrandedTypes';
import {createMessageID, createUserID} from '../../BrandedTypes';
import {SCHEDULED_MESSAGE_TTL_SECONDS} from '../../channel/services/ScheduledMessageService';
import {createRequestCache} from '../../middleware/RequestCacheMiddleware';
import {ScheduledMessageRepository} from '../../user/repositories/ScheduledMessageRepository';
import type {WorkerDependencies} from '../WorkerDependencies';

interface WorkerLogger {
	debug(message: string, extra?: object): void;
	info(message: string, extra?: object): void;
	warn(message: string, extra?: object): void;
	error(message: string, extra?: object): void;
}

export interface SendScheduledMessageParams {
	userId: string;
	scheduledMessageId: string;
	expectedScheduledAt: string;
}

export class ScheduledMessageExecutor {
	constructor(
		private readonly deps: WorkerDependencies,
		private readonly logger: WorkerLogger,
		private readonly scheduledMessageRepository: ScheduledMessageRepository = new ScheduledMessageRepository(),
	) {}

	async execute(params: SendScheduledMessageParams): Promise<void> {
		const userId = this.parseUserID(params.userId);
		const scheduledMessageId = this.parseMessageID(params.scheduledMessageId);
		if (!userId || !scheduledMessageId) {
			this.logger.warn('Malformed scheduled message job payload', {payload: params});
			return;
		}
		const expectedScheduledAt = this.parseScheduledAt(params.expectedScheduledAt);
		if (!expectedScheduledAt) {
			this.logger.warn('Invalid expectedScheduledAt for scheduled message job', {payload: params});
			return;
		}
		const scheduledMessage = await this.scheduledMessageRepository.getScheduledMessage(userId, scheduledMessageId);
		if (!scheduledMessage) {
			this.logger.info('Scheduled message not found, skipping', {userId, scheduledMessageId});
			return;
		}
		if (scheduledMessage.status !== 'pending') {
			this.logger.info('Scheduled message already processed', {
				scheduledMessageId,
				status: scheduledMessage.status,
			});
			return;
		}
		if (scheduledMessage.scheduledAt.toISOString() !== expectedScheduledAt.toISOString()) {
			this.logger.info('Scheduled message time mismatch, skipping stale job', {
				scheduledMessageId,
				expected: expectedScheduledAt.toISOString(),
				actual: scheduledMessage.scheduledAt.toISOString(),
			});
			return;
		}
		const user = await this.deps.userRepository.findUnique(userId);
		if (!user) {
			await this.markInvalid(userId, scheduledMessageId, 'User not found');
			return;
		}
		const messageRequest = scheduledMessage.parseToMessageRequest();
		const requestCache = createRequestCache();
		try {
			await this.deps.channelService.messages.send.validateMessageCanBeSent({
				user,
				channelId: scheduledMessage.channelId,
				data: messageRequest,
			});
			await this.deps.channelService.messages.send.sendMessage({
				user,
				channelId: scheduledMessage.channelId,
				data: messageRequest,
				requestCache,
			});
			await this.scheduledMessageRepository.deleteScheduledMessage(userId, scheduledMessageId);
			this.logger.info('Scheduled message sent successfully', {scheduledMessageId, userId});
		} catch (error) {
			const reason = error instanceof Error ? error.message : 'Failed to send scheduled message';
			this.logger.warn('Marking scheduled message invalid', {scheduledMessageId, userId, reason});
			await this.markInvalid(userId, scheduledMessageId, reason);
		} finally {
			requestCache.clear();
		}
	}

	private async markInvalid(userId: UserID, scheduledMessageId: MessageID, reason: string): Promise<void> {
		try {
			await this.scheduledMessageRepository.markInvalid(
				userId,
				scheduledMessageId,
				reason,
				SCHEDULED_MESSAGE_TTL_SECONDS,
			);
		} catch (error) {
			this.logger.error('Failed to mark scheduled message invalid', {error, scheduledMessageId});
		}
	}

	private parseUserID(value: string): UserID | null {
		try {
			return createUserID(BigInt(value));
		} catch {
			return null;
		}
	}

	private parseMessageID(value: string): MessageID | null {
		try {
			return createMessageID(BigInt(value));
		} catch {
			return null;
		}
	}

	private parseScheduledAt(value: string): Date | null {
		const date = new Date(value);
		return Number.isNaN(date.getTime()) ? null : date;
	}
}
