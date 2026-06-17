// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';

const WEBHOOK_INFLIGHT_TTL_SECONDS = 10 * 60;
const WEBHOOK_PROCESSED_TTL_SECONDS = 7 * 24 * 60 * 60;

type WebhookClaimResult = 'claimed' | 'already_processed' | 'in_flight';

function key(eventId: string): string {
	return `stripe-webhook:${eventId}`;
}

export class BillingWebhookEventRepository {
	constructor(private readonly kv: IKVProvider) {}

	async tryClaim(eventId: string): Promise<WebhookClaimResult> {
		const k = key(eventId);
		const claimed = await this.kv.setnx(k, 'in_flight', WEBHOOK_INFLIGHT_TTL_SECONDS);
		if (claimed) {
			return 'claimed';
		}
		const current = await this.kv.get(k);
		return current === 'processed' ? 'already_processed' : 'in_flight';
	}

	async markProcessed(eventId: string): Promise<void> {
		await this.kv.setex(key(eventId), WEBHOOK_PROCESSED_TTL_SECONDS, 'processed');
	}

	async releaseClaim(eventId: string): Promise<void> {
		await this.kv.del(key(eventId));
	}
}
