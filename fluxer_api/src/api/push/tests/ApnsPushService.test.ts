// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {ApnsPushServiceTestHooks} from '../ApnsPushService';

describe('ApnsPushService', () => {
	it('builds modern APNs alert payloads for chat messages', () => {
		const payload = ApnsPushServiceTestHooks.buildApnsPayload({
			tag: 'channel:123:456',
			image_url: 'https://cdn.example/image.png',
			data: {
				channel_id: '123',
				message_id: '456',
				notification_tag: 'channel:123',
				badge_count: 7,
				url: '/channels/@me/123/456',
			},
			notification: {
				title: 'Alice',
				body: 'Hello',
			},
		});
		expect(payload).toMatchObject({
			channel_id: '123',
			message_id: '456',
			title: 'Alice',
			body: 'Hello',
			url: '/channels/@me/123/456',
			image_url: 'https://cdn.example/image.png',
			aps: {
				alert: {title: 'Alice', body: 'Hello'},
				sound: 'default',
				badge: 7,
				'thread-id': 'channel:123',
				category: 'FLUXER_MESSAGE',
				'interruption-level': 'active',
				'mutable-content': 1,
			},
		});
	});
	it('builds silent APNs clear payloads with badge when badge_count is present', () => {
		const payload = ApnsPushServiceTestHooks.buildApnsPayload({
			type: 'notification_clear',
			action: 'clear_channel',
			data: {
				channel_id: '123',
				message_id: '456',
				badge_count: 0,
			},
		});
		expect(payload).toMatchObject({
			type: 'notification_clear',
			action: 'clear_channel',
			channel_id: '123',
			message_id: '456',
			badge_count: 0,
			aps: {
				'content-available': 1,
				badge: 0,
			},
		});
		expect(payload.aps).not.toHaveProperty('alert');
		expect(payload.aps).not.toHaveProperty('sound');
	});
	it('uses APNs push-type and priority headers that match alert versus background delivery', () => {
		const alertHeaders = ApnsPushServiceTestHooks.buildApnsHeaders({
			providerToken: 'provider-token',
			topic: 'com.fluxer',
			payload: {
				tag: 'channel:123:456',
				data: {
					message_id: '456',
				},
			},
		});
		const clearHeaders = ApnsPushServiceTestHooks.buildApnsHeaders({
			providerToken: 'provider-token',
			topic: 'com.fluxer',
			payload: {
				type: 'notification_clear',
				tag: 'channel:123',
				data: {},
			},
		});
		expect(alertHeaders).toMatchObject({
			authorization: 'bearer provider-token',
			'apns-topic': 'com.fluxer',
			'apns-push-type': 'alert',
			'apns-priority': '10',
			'apns-collapse-id': 'channel:123:456',
		});
		expect(clearHeaders).toMatchObject({
			'apns-push-type': 'background',
			'apns-priority': '5',
			'apns-collapse-id': 'channel:123',
		});
	});
	it('marks only permanent APNs token failures as subscription deletion signals', () => {
		expect(ApnsPushServiceTestHooks.isPermanentApnsFailure(410, 'Unregistered')).toBe(true);
		expect(ApnsPushServiceTestHooks.isPermanentApnsFailure(400, 'BadDeviceToken')).toBe(true);
		expect(ApnsPushServiceTestHooks.isPermanentApnsFailure(400, 'DeviceTokenNotForTopic')).toBe(true);
		expect(ApnsPushServiceTestHooks.isPermanentApnsFailure(403, 'ExpiredProviderToken')).toBe(false);
		expect(ApnsPushServiceTestHooks.isPermanentApnsFailure(500, 'InternalServerError')).toBe(false);
	});
});
