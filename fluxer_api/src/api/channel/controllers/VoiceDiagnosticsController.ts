// SPDX-License-Identifier: AGPL-3.0-or-later

import {UserFlags} from '@fluxer/constants/src/UserConstants';
import {MissingPermissionsError} from '@fluxer/errors/src/domains/core/MissingPermissionsError';
import {
	VoiceDebugLoggingEventsBodySchema,
	VoiceDebugLoggingToggleBodySchema,
} from '@fluxer/schema/src/domains/channel/ChannelRequestSchemas';
import {
	VoiceDebugLoggingEventsResponse,
	VoiceDebugLoggingStatusResponse,
} from '@fluxer/schema/src/domains/channel/ChannelSchemas';
import {ChannelIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import type {Context} from 'hono';
import {createChannelID} from '../../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp, HonoEnv} from '../../types/HonoEnv';
import {Validator} from '../../Validator';
import {VoiceDiagnosticsService} from '../services/VoiceDiagnosticsService';

function makeVoiceDiagnosticsService(ctx: Context<HonoEnv>): VoiceDiagnosticsService {
	return new VoiceDiagnosticsService(
		ctx.get('cacheService'),
		ctx.get('channelService'),
		ctx.get('gatewayService'),
		ctx.get('storageService'),
	);
}

export function VoiceDiagnosticsController(app: HonoApp) {
	app.get(
		'/channels/:channel_id/voice-debug-logging/session',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_VOICE_DEBUG_LOGGING_STATUS),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		OpenAPI({
			operationId: 'get_voice_debug_logging_status',
			summary: 'Get voice debug logging status',
			description:
				'Returns whether staff-enabled voice debug logging is active for this channel. Clients poll this while connected to decide whether to upload diagnostics.',
			responseSchema: VoiceDebugLoggingStatusResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const service = makeVoiceDiagnosticsService(ctx);
			return ctx.json(await service.getStatus({userId, channelId}));
		},
	);
	app.put(
		'/channels/:channel_id/voice-debug-logging/session',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_VOICE_DEBUG_LOGGING_TOGGLE),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		Validator('json', VoiceDebugLoggingToggleBodySchema),
		OpenAPI({
			operationId: 'set_voice_debug_logging_status',
			summary: 'Toggle voice debug logging',
			description:
				'Allows staff to start or stop a channel-scoped voice debug logging session. Non-staff users cannot activate or stop sessions.',
			responseSchema: VoiceDebugLoggingStatusResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const user = ctx.get('user');
			if ((user.flags & UserFlags.STAFF) === 0n) {
				throw new MissingPermissionsError();
			}
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const {enabled, duration_ms} = ctx.req.valid('json');
			const service = makeVoiceDiagnosticsService(ctx);
			return ctx.json(await service.setSession({userId: user.id, channelId, enabled, durationMs: duration_ms}));
		},
	);
	app.post(
		'/channels/:channel_id/voice-debug-logging/events',
		RateLimitMiddleware(RateLimitConfigs.CHANNEL_VOICE_DEBUG_LOGGING_EVENTS),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		Validator('json', VoiceDebugLoggingEventsBodySchema),
		OpenAPI({
			operationId: 'upload_voice_debug_logging_events',
			summary: 'Upload voice debug logging events',
			description:
				'Uploads a small batch of client voice diagnostics events for an active staff-enabled debug logging session.',
			responseSchema: VoiceDebugLoggingEventsResponse,
			statusCode: 200,
			security: ['bearerToken', 'sessionToken'],
			tags: 'Channels',
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const channelId = createChannelID(ctx.req.valid('param').channel_id);
			const body = ctx.req.valid('json');
			const service = makeVoiceDiagnosticsService(ctx);
			return ctx.json(await service.ingestEvents({userId, channelId, body}));
		},
	);
}
