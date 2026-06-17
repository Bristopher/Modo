// SPDX-License-Identifier: AGPL-3.0-or-later

import {Readable} from 'node:stream';
import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	VoiceDiagnosticsObjectListResponse,
	VoiceDiagnosticsQueryRequest,
} from '@fluxer/schema/src/domains/admin/AdminVoiceSchemas';
import {VOICE_DIAGNOSTICS_BUCKET, VoiceDiagnosticsService} from '../../channel/services/VoiceDiagnosticsService';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function VoiceDiagnosticsAdminController(app: HonoApp) {
	app.get(
		'/admin/voice/diagnostics/objects',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_DIAGNOSTICS_VIEW),
		Validator('query', VoiceDiagnosticsQueryRequest),
		OpenAPI({
			operationId: 'list_voice_diagnostics_objects',
			summary: 'List voice diagnostics objects',
			responseSchema: VoiceDiagnosticsObjectListResponse,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Lists raw voice diagnostics NDJSON S3 objects for a channel and time range. Requires VOICE_DIAGNOSTICS_VIEW permission.',
		}),
		async (ctx) => {
			const query = ctx.req.valid('query');
			const service = new VoiceDiagnosticsService(
				ctx.get('cacheService'),
				ctx.get('channelService'),
				ctx.get('gatewayService'),
				ctx.get('storageService'),
			);
			const objects = await service.listObjects({
				channelId: query.channel_id,
				startMs: query.start_ms,
				endMs: query.end_ms,
				sessionId: query.session_id,
				limitObjects: query.limit_objects,
			});
			return ctx.json({bucket: VOICE_DIAGNOSTICS_BUCKET, objects});
		},
	);
	app.get(
		'/admin/voice/diagnostics/raw',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_LOOKUP),
		requireAdminACL(AdminACLs.VOICE_DIAGNOSTICS_VIEW),
		Validator('query', VoiceDiagnosticsQueryRequest),
		OpenAPI({
			operationId: 'stream_voice_diagnostics_raw',
			summary: 'Stream raw voice diagnostics',
			responseSchema: null,
			statusCode: 200,
			security: 'adminApiKey',
			tags: 'Admin',
			description:
				'Streams matching voice diagnostics NDJSON objects for local processing. Requires VOICE_DIAGNOSTICS_VIEW permission.',
		}),
		async (ctx) => {
			const query = ctx.req.valid('query');
			const service = new VoiceDiagnosticsService(
				ctx.get('cacheService'),
				ctx.get('channelService'),
				ctx.get('gatewayService'),
				ctx.get('storageService'),
			);
			const objects = await service.listObjects({
				channelId: query.channel_id,
				startMs: query.start_ms,
				endMs: query.end_ms,
				sessionId: query.session_id,
				limitObjects: query.limit_objects,
			});
			const stream = service.createRawObjectStream(objects);
			return new Response(Readable.toWeb(stream) as ReadableStream, {
				status: 200,
				headers: {
					'Content-Type': 'application/x-ndjson',
					'Cache-Control': 'no-store, private',
					'X-Fluxer-Diagnostics-Bucket': VOICE_DIAGNOSTICS_BUCKET,
					'X-Fluxer-Diagnostics-Object-Count': objects.length.toString(),
				},
			});
		},
	);
}
