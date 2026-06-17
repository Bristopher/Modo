// SPDX-License-Identifier: AGPL-3.0-or-later

export type PneumaticPostDeliveryStatus = 'claimed' | 'failed' | 'sent';

export interface PneumaticPostDeliveryRow {
	dispatch_key: string;
	user_id: bigint;
	status: PneumaticPostDeliveryStatus;
	claimed_at: Date;
	sent_at: Date | null;
	channel_id: bigint | null;
	message_id: bigint | null;
	locale: string | null;
	error_message: string | null;
}

export const PNEUMATIC_POST_DELIVERY_COLUMNS = [
	'user_id',
	'dispatch_key',
	'status',
	'claimed_at',
	'sent_at',
	'channel_id',
	'message_id',
	'locale',
	'error_message',
] as const satisfies ReadonlyArray<keyof PneumaticPostDeliveryRow>;
