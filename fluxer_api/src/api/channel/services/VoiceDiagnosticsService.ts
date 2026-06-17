// SPDX-License-Identifier: AGPL-3.0-or-later

import {Readable} from 'node:stream';
import {Permissions} from '@fluxer/constants/src/ChannelConstants';
import {MissingPermissionsError} from '@fluxer/errors/src/domains/core/MissingPermissionsError';
import type {
	VoiceDebugLoggingEventSchema,
	VoiceDebugLoggingEventsBodySchema,
} from '@fluxer/schema/src/domains/channel/ChannelRequestSchemas';
import type {
	VoiceDebugLoggingEventsResponse,
	VoiceDebugLoggingStatusResponse,
} from '@fluxer/schema/src/domains/channel/ChannelSchemas';
import type {ICacheService} from '@pkgs/cache/src/ICacheService';
import type {ChannelID, UserID} from '../../BrandedTypes';
import type {IGatewayService} from '../../infrastructure/IGatewayService';
import type {IStorageService} from '../../infrastructure/IStorageService';
import type {ChannelService} from './ChannelService';

export const VOICE_DIAGNOSTICS_BUCKET = 'fluxer-voice-diagnostics';
const VOICE_DEBUG_LOGGING_POLL_INTERVAL_MS = 10000;
const VOICE_DEBUG_LOGGING_UPLOAD_INTERVAL_MS = 2000;

const DEFAULT_SESSION_DURATION_MS = 60 * 60 * 1000;
const MAX_SESSION_DURATION_MS = 4 * 60 * 60 * 1000;
const NDJSON_CONTENT_TYPE = 'application/x-ndjson';
const TEXT_ENCODER = new TextEncoder();

interface ActiveVoiceDebugLoggingSession {
	session_id: string;
	channel_id: string;
	activated_by_user_id: string;
	started_at_ms: number;
	expires_at_ms: number;
}

interface VoiceDiagnosticsObject {
	key: string;
	session_id: string;
	start_ns: string;
	end_ns: string;
	last_modified: string | null;
}

function sessionCacheKey(channelId: ChannelID): string {
	return `voice_debug_logging:channel:${channelId.toString()}`;
}

function createId(prefix: string): string {
	const randomUUID = globalThis.crypto?.randomUUID?.bind(globalThis.crypto);
	if (randomUUID) return `${prefix}_${randomUUID()}`;
	return `${prefix}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2)}`;
}

function datePartFromMs(timestampMs: number): string {
	return new Date(timestampMs).toISOString().slice(0, 10);
}

function sanitizeKeyPart(value: string): string {
	return value.replace(/[^A-Za-z0-9_.:-]/g, '_').slice(0, 256) || 'unknown';
}

function minMaxEventTimestampNs(events: ReadonlyArray<VoiceDebugLoggingEventSchema>): {
	startNs: bigint;
	endNs: bigint;
} {
	let startNs: bigint | null = null;
	let endNs: bigint | null = null;
	for (const event of events) {
		const timestampNs = BigInt(event.timestamp_ns);
		if (startNs === null || timestampNs < startNs) startNs = timestampNs;
		if (endNs === null || timestampNs > endNs) endNs = timestampNs;
	}
	return {startNs: startNs ?? 0n, endNs: endNs ?? 0n};
}

function buildDiagnosticsKey(params: {
	channelId: ChannelID;
	sessionId: string;
	userId: UserID;
	connectionId?: string;
	participantIdentity?: string;
	startNs: bigint;
	endNs: bigint;
}): string {
	const timestampMs = Number(params.startNs / 1000000n);
	const date = datePartFromMs(timestampMs > 0 ? timestampMs : Date.now());
	return [
		`channel_id=${params.channelId.toString()}`,
		`date=${date}`,
		`session_id=${sanitizeKeyPart(params.sessionId)}`,
		`start_ns=${params.startNs.toString()}_end_ns=${params.endNs.toString()}`,
		`participant=${sanitizeKeyPart(params.userId.toString())}`,
		`connection=${sanitizeKeyPart(params.connectionId ?? 'unknown')}`,
		`identity=${sanitizeKeyPart(params.participantIdentity ?? 'unknown')}`,
		`batch=${createId('batch')}.ndjson`,
	].join('/');
}

function parseDiagnosticsKey(key: string): VoiceDiagnosticsObject | null {
	const match = key.match(/^channel_id=[^/]+\/date=[^/]+\/session_id=([^/]+)\/start_ns=([0-9]+)_end_ns=([0-9]+)\//);
	if (!match) return null;
	const [, sessionId, startNs, endNs] = match;
	if (!sessionId || !startNs || !endNs) return null;
	return {
		key,
		session_id: sessionId,
		start_ns: startNs,
		end_ns: endNs,
		last_modified: null,
	};
}

function buildDatePrefixes(params: {
	channelId: string;
	startMs: number;
	endMs: number;
	sessionId?: string;
}): Array<string> {
	const prefixes: Array<string> = [];
	const cursor = new Date(params.startMs);
	cursor.setUTCHours(0, 0, 0, 0);
	const end = new Date(params.endMs);
	end.setUTCHours(0, 0, 0, 0);
	while (cursor.getTime() <= end.getTime()) {
		const base = `channel_id=${params.channelId}/date=${datePartFromMs(cursor.getTime())}/`;
		prefixes.push(params.sessionId ? `${base}session_id=${sanitizeKeyPart(params.sessionId)}/` : base);
		cursor.setUTCDate(cursor.getUTCDate() + 1);
	}
	return prefixes;
}

async function assertVoiceChannelAccess(params: {
	channelService: ChannelService;
	gatewayService: IGatewayService;
	userId: UserID;
	channelId: ChannelID;
}): Promise<void> {
	const channel = await params.channelService.channelData.operations.getChannel({
		userId: params.userId,
		channelId: params.channelId,
	});
	if (!channel.guildId) return;
	const hasConnect = await params.gatewayService.checkPermission({
		guildId: channel.guildId,
		channelId: params.channelId,
		userId: params.userId,
		permission: Permissions.CONNECT,
	});
	if (!hasConnect) {
		throw new MissingPermissionsError();
	}
}

export class VoiceDiagnosticsService {
	constructor(
		private readonly cacheService: ICacheService,
		private readonly channelService: ChannelService,
		private readonly gatewayService: IGatewayService,
		private readonly storageService: IStorageService,
	) {}

	private async getActiveSession(channelId: ChannelID): Promise<ActiveVoiceDebugLoggingSession | null> {
		const session = await this.cacheService.get<ActiveVoiceDebugLoggingSession>(sessionCacheKey(channelId));
		if (!session) return null;
		if (session.expires_at_ms <= Date.now()) {
			await this.cacheService.delete(sessionCacheKey(channelId));
			return null;
		}
		return session;
	}

	private statusFromSession(session: ActiveVoiceDebugLoggingSession | null): VoiceDebugLoggingStatusResponse {
		return {
			active: session !== null,
			session_id: session?.session_id ?? null,
			activated_by_user_id: session?.activated_by_user_id ?? null,
			started_at_ms: session?.started_at_ms ?? null,
			expires_at_ms: session?.expires_at_ms ?? null,
			poll_interval_ms: VOICE_DEBUG_LOGGING_POLL_INTERVAL_MS,
			upload_interval_ms: VOICE_DEBUG_LOGGING_UPLOAD_INTERVAL_MS,
		};
	}

	async getStatus(params: {userId: UserID; channelId: ChannelID}): Promise<VoiceDebugLoggingStatusResponse> {
		await assertVoiceChannelAccess({
			channelService: this.channelService,
			gatewayService: this.gatewayService,
			userId: params.userId,
			channelId: params.channelId,
		});
		return this.statusFromSession(await this.getActiveSession(params.channelId));
	}

	async setSession(params: {
		userId: UserID;
		channelId: ChannelID;
		enabled: boolean;
		durationMs?: number;
	}): Promise<VoiceDebugLoggingStatusResponse> {
		await assertVoiceChannelAccess({
			channelService: this.channelService,
			gatewayService: this.gatewayService,
			userId: params.userId,
			channelId: params.channelId,
		});
		const key = sessionCacheKey(params.channelId);
		if (!params.enabled) {
			await this.cacheService.delete(key);
			return this.statusFromSession(null);
		}
		const now = Date.now();
		const durationMs = Math.min(params.durationMs ?? DEFAULT_SESSION_DURATION_MS, MAX_SESSION_DURATION_MS);
		const session: ActiveVoiceDebugLoggingSession = {
			session_id: createId('voice_debug'),
			channel_id: params.channelId.toString(),
			activated_by_user_id: params.userId.toString(),
			started_at_ms: now,
			expires_at_ms: now + durationMs,
		};
		await this.cacheService.set(key, session, Math.ceil(durationMs / 1000));
		return this.statusFromSession(session);
	}

	async ingestEvents(params: {
		userId: UserID;
		channelId: ChannelID;
		body: VoiceDebugLoggingEventsBodySchema;
	}): Promise<VoiceDebugLoggingEventsResponse> {
		await assertVoiceChannelAccess({
			channelService: this.channelService,
			gatewayService: this.gatewayService,
			userId: params.userId,
			channelId: params.channelId,
		});
		const session = await this.getActiveSession(params.channelId);
		if (!session || session.session_id !== params.body.session_id) {
			return {accepted: false, active: session !== null, stored_event_count: 0};
		}
		const {startNs, endNs} = minMaxEventTimestampNs(params.body.events);
		const key = buildDiagnosticsKey({
			channelId: params.channelId,
			sessionId: session.session_id,
			userId: params.userId,
			connectionId: params.body.connection_id,
			participantIdentity: params.body.participant_identity,
			startNs,
			endNs,
		});
		const serverReceivedAtMs = Date.now();
		const serverReceivedMonotonicNs = process.hrtime.bigint().toString();
		const lines = params.body.events.map((event) =>
			JSON.stringify({
				schema_version: 1,
				server_received_at_ms: serverReceivedAtMs,
				server_received_monotonic_ns: serverReceivedMonotonicNs,
				channel_id: params.channelId.toString(),
				session_id: session.session_id,
				activated_by_user_id: session.activated_by_user_id,
				participant_user_id: params.userId.toString(),
				connection_id: params.body.connection_id ?? null,
				participant_identity: params.body.participant_identity ?? null,
				event,
			}),
		);
		await this.storageService.uploadObject({
			bucket: VOICE_DIAGNOSTICS_BUCKET,
			key,
			body: TEXT_ENCODER.encode(`${lines.join('\n')}\n`),
			contentType: NDJSON_CONTENT_TYPE,
		});
		return {accepted: true, active: true, stored_event_count: params.body.events.length};
	}

	async listObjects(params: {
		channelId: string;
		startMs: number;
		endMs: number;
		sessionId?: string;
		limitObjects: number;
	}): Promise<Array<VoiceDiagnosticsObject>> {
		const startNs = BigInt(Math.floor(params.startMs)) * 1000000n;
		const endNs = BigInt(Math.floor(params.endMs)) * 1000000n;
		const prefixes = buildDatePrefixes({
			channelId: params.channelId,
			startMs: params.startMs,
			endMs: params.endMs,
			sessionId: params.sessionId,
		});
		const listed = await Promise.all(
			prefixes.map((prefix) => this.storageService.listObjects({bucket: VOICE_DIAGNOSTICS_BUCKET, prefix})),
		);
		return listed
			.flat()
			.flatMap((object): Array<VoiceDiagnosticsObject> => {
				const parsed = parseDiagnosticsKey(object.key);
				if (!parsed) return [];
				return [
					{
						...parsed,
						last_modified: object.lastModified?.toISOString() ?? null,
					},
				];
			})
			.filter((object) => {
				const objectStartNs = BigInt(object.start_ns);
				const objectEndNs = BigInt(object.end_ns);
				return objectEndNs >= startNs && objectStartNs <= endNs;
			})
			.sort((a, b) => {
				const delta = BigInt(a.start_ns) - BigInt(b.start_ns);
				if (delta < 0n) return -1;
				if (delta > 0n) return 1;
				return a.key.localeCompare(b.key);
			})
			.slice(0, params.limitObjects);
	}

	createRawObjectStream(objects: ReadonlyArray<VoiceDiagnosticsObject>): Readable {
		const storageService = this.storageService;
		async function* streamObjects(): AsyncGenerator<Uint8Array> {
			for (const object of objects) {
				const stream = await storageService.streamObject({
					bucket: VOICE_DIAGNOSTICS_BUCKET,
					key: object.key,
				});
				if (!stream) continue;
				for await (const chunk of stream.body) {
					yield typeof chunk === 'string' ? TEXT_ENCODER.encode(chunk) : new Uint8Array(chunk);
				}
				yield TEXT_ENCODER.encode('\n');
			}
		}
		return Readable.from(streamObjects());
	}
}
