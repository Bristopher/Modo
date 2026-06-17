// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageID, UserID} from '../../BrandedTypes';
import {deleteOneOrMany, fetchMany, fetchOne, upsertOne} from '../../database/CassandraQueryExecution';
import {Db} from '../../database/CassandraTypes';
import type {ScheduledMessageRow} from '../../database/types/UserTypes';
import {ScheduledMessage} from '../../models/ScheduledMessage';
import {ScheduledMessages} from '../../Tables';

export class ScheduledMessageRepository {
	private readonly fetchCql = ScheduledMessages.selectCql({
		where: [ScheduledMessages.where.eq('user_id')],
	});

	async listScheduledMessages(userId: UserID, limit: number = 25): Promise<Array<ScheduledMessage>> {
		const rows = await fetchMany<ScheduledMessageRow>(this.fetchCql, {
			user_id: userId,
		});
		const messages = rows.map((row) => ScheduledMessage.fromRow(row));
		return messages.sort((a, b) => (b.id > a.id ? 1 : a.id > b.id ? -1 : 0)).slice(0, limit);
	}

	async getScheduledMessage(userId: UserID, scheduledMessageId: MessageID): Promise<ScheduledMessage | null> {
		const row = await fetchOne<ScheduledMessageRow>(
			ScheduledMessages.selectCql({
				where: [ScheduledMessages.where.eq('user_id'), ScheduledMessages.where.eq('scheduled_message_id')],
			}),
			{
				user_id: userId,
				scheduled_message_id: scheduledMessageId,
			},
		);
		return row ? ScheduledMessage.fromRow(row) : null;
	}

	async upsertScheduledMessage(message: ScheduledMessage, ttlSeconds: number): Promise<void> {
		await upsertOne(ScheduledMessages.upsertAllWithTtl(message.toRow(), ttlSeconds));
	}

	async deleteScheduledMessage(userId: UserID, scheduledMessageId: MessageID): Promise<void> {
		await deleteOneOrMany(
			ScheduledMessages.deleteByPk({
				user_id: userId,
				scheduled_message_id: scheduledMessageId,
			}),
		);
	}

	async markInvalid(userId: UserID, scheduledMessageId: MessageID, reason: string, ttlSeconds: number): Promise<void> {
		await upsertOne(
			ScheduledMessages.patchByPkWithTtl(
				{
					user_id: userId,
					scheduled_message_id: scheduledMessageId,
				},
				{
					status: Db.set('invalid'),
					status_reason: Db.set(reason),
					invalidated_at: Db.set(new Date()),
				},
				ttlSeconds,
			),
		);
	}
}
