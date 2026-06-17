// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ChannelID, MessageID, UserID} from '../BrandedTypes';
import {executeQuery, fetchOne, upsertOne} from '../database/CassandraQueryExecution';
import {Db} from '../database/CassandraTypes';
import type {PneumaticPostDeliveryRow} from '../database/types/PneumaticPostTypes';
import {PneumaticPostDeliveries} from '../Tables';

const FIND_DELIVERY = PneumaticPostDeliveries.select({
	where: [PneumaticPostDeliveries.where.eq('user_id'), PneumaticPostDeliveries.where.eq('dispatch_key')],
	limit: 1,
});

interface ConditionalInsertResult {
	'[applied]': boolean;
}

export interface PneumaticPostDeliveryRecord {
	dispatchKey: string;
	userId: UserID;
	claimedAt: Date;
	locale: string;
}

interface PneumaticPostSentDeliveryRecord extends PneumaticPostDeliveryRecord {
	channelId: ChannelID;
	messageId: MessageID;
	sentAt: Date;
}

interface PneumaticPostFailedDeliveryRecord extends PneumaticPostDeliveryRecord {
	errorMessage: string;
}

export class PneumaticPostRepository {
	async findDelivery(userId: UserID, dispatchKey: string): Promise<PneumaticPostDeliveryRow | null> {
		return await fetchOne<PneumaticPostDeliveryRow>(
			FIND_DELIVERY.bind({
				user_id: userId,
				dispatch_key: dispatchKey,
			}),
		);
	}

	async tryCreateDeliveryClaim(record: PneumaticPostDeliveryRecord): Promise<boolean> {
		const row: PneumaticPostDeliveryRow = {
			user_id: record.userId,
			dispatch_key: record.dispatchKey,
			status: 'claimed',
			claimed_at: record.claimedAt,
			sent_at: null,
			channel_id: null,
			message_id: null,
			locale: record.locale,
			error_message: null,
		};
		const [result] = await executeQuery<ConditionalInsertResult>(PneumaticPostDeliveries.insertIfNotExists(row));
		if (!result || typeof result['[applied]'] !== 'boolean') {
			throw new Error('Unexpected database response for Pneumatic Post delivery claim');
		}
		return result['[applied]'];
	}

	async markDeliverySent(record: PneumaticPostSentDeliveryRecord): Promise<void> {
		await upsertOne(
			PneumaticPostDeliveries.patchByPk(
				{user_id: record.userId, dispatch_key: record.dispatchKey},
				{
					status: Db.set('sent'),
					claimed_at: Db.set(record.claimedAt),
					sent_at: Db.set(record.sentAt),
					channel_id: Db.set(record.channelId),
					message_id: Db.set(record.messageId),
					locale: Db.set(record.locale),
					error_message: Db.clear(),
				},
			),
		);
	}

	async markDeliveryFailed(record: PneumaticPostFailedDeliveryRecord): Promise<void> {
		await upsertOne(
			PneumaticPostDeliveries.patchByPk(
				{user_id: record.userId, dispatch_key: record.dispatchKey},
				{
					status: Db.set('failed'),
					claimed_at: Db.set(record.claimedAt),
					sent_at: Db.clear(),
					channel_id: Db.clear(),
					message_id: Db.clear(),
					locale: Db.set(record.locale),
					error_message: Db.set(record.errorMessage),
				},
			),
		);
	}
}
