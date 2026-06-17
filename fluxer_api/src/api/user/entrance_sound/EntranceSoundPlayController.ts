// SPDX-License-Identifier: AGPL-3.0-or-later

import {ChannelIdParam} from '@fluxer/schema/src/domains/common/CommonParamSchemas';
import {EntranceSoundPlayRequest} from '@fluxer/schema/src/domains/user/EntranceSoundSchemas';
import {createChannelID, createEntranceSoundID} from '../../BrandedTypes';
import {DefaultUserOnly, LoginRequired} from '../../middleware/AuthMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';
import type {EntranceSoundPlayService} from './EntranceSoundPlayService';

export function EntranceSoundPlayController(app: HonoApp) {
	app.post(
		'/voice/channels/:channel_id/entrance-sound',
		RateLimitMiddleware(RateLimitConfigs.VOICE_ENTRANCE_SOUND_PLAY),
		LoginRequired,
		DefaultUserOnly,
		Validator('param', ChannelIdParam),
		Validator('json', EntranceSoundPlayRequest),
		OpenAPI({
			operationId: 'play_entrance_sound',
			summary: 'Play an entrance sound in a voice channel',
			description:
				'Requests that the API fan out an ENTRANCE_SOUND_PLAY gateway event to every other user currently connected to the voice channel. The other clients then fetch the audio from CDN and play it locally; no LiveKit track is published.',
			requestSchema: EntranceSoundPlayRequest,
			responseSchema: null,
			statusCode: 204,
			security: ['bearerToken', 'sessionToken'],
			tags: ['Voice'],
		}),
		async (ctx) => {
			const userId = ctx.get('user').id;
			const channelId = createChannelID(BigInt(ctx.req.valid('param').channel_id));
			const soundId = createEntranceSoundID(BigInt(ctx.req.valid('json').sound_id));
			const service: EntranceSoundPlayService = ctx.get('entranceSoundPlayService');
			await service.play({userId, channelId, soundId});
			return ctx.body(null, 204);
		},
	);
}
