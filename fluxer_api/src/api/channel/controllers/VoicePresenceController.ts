// SPDX-License-Identifier: AGPL-3.0-or-later

import {VoicePresenceHeartbeatBodySchema} from '@fluxer/schema/src/domains/channel/ChannelRequestSchemas';
import {
	VoicePresenceHeartbeatEndResponse,
	VoicePresenceHeartbeatResponse,
} from '@fluxer/schema/src/domains/channel/ChannelSchemas';
import {ChannelIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {createChannelID} from '../../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';
import {VoicePresenceHeartbeatStore} from '../../voice/VoicePresenceHeartbeatStore';

export function VoicePresenceController(app: HonoApp) {
	app.post(
		'/channels/:channel_id/voice-presence/heartbeat',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_VOICE_PRESENCE_HEARTBEAT),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		Validator('json', VoicePresenceHeartbeatBodySchema),
		OpenAPI({
			operationId: 'heartbeat_voice_presence',
			summary: 'Heartbeat voice presence',
			description:
				'Refreshes the current user voice presence marker for v2 voice reconciliation. Clients call this while connected to voice.',
			responseSchema: VoicePresenceHeartbeatResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const body = ctx.req.valid('json');
			const store = new VoicePresenceHeartbeatStore(ctx.get('apiContext').services.kv);
			const heartbeat = await store.recordHeartbeat({
				channelId,
				userId,
				connectionId: body.connection_id,
			});
			return ctx.json({
				ok: true,
				heartbeat_interval_ms: heartbeat.heartbeatIntervalMs,
				heartbeat_ttl_ms: heartbeat.heartbeatTtlMs,
				expires_at_ms: heartbeat.expiresAtMs,
			});
		},
	);
	app.delete(
		'/channels/:channel_id/voice-presence/heartbeat',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_VOICE_PRESENCE_HEARTBEAT),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		Validator('json', VoicePresenceHeartbeatBodySchema),
		OpenAPI({
			operationId: 'end_voice_presence_heartbeat',
			summary: 'End voice presence heartbeat',
			description:
				'Clears the current user active v2 voice presence marker for a voice connection while preserving v2 enrollment for fast reconciliation.',
			responseSchema: VoicePresenceHeartbeatEndResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const body = ctx.req.valid('json');
			const store = new VoicePresenceHeartbeatStore(ctx.get('apiContext').services.kv);
			await store.markHeartbeatEnded({
				channelId,
				userId,
				connectionId: body.connection_id,
			});
			return ctx.json({ok: true});
		},
	);
}
