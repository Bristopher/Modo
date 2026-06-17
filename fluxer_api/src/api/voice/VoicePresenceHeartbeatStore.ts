// SPDX-License-Identifier: AGPL-3.0-or-later

import {createHash} from 'node:crypto';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import type {ChannelID, UserID} from '../BrandedTypes';

export type VoicePresenceHeartbeatState = 'active' | 'expired' | 'legacy';

interface VoicePresenceHeartbeatParams {
	channelId: ChannelID;
	userId: UserID;
	connectionId: string;
}

interface VoicePresenceHeartbeatResult {
	heartbeatIntervalMs: number;
	heartbeatTtlMs: number;
	expiresAtMs: number;
}

const VOICE_PRESENCE_HEARTBEAT_INTERVAL_MS = 15000;
const VOICE_PRESENCE_HEARTBEAT_TTL_SECONDS = 45;
const VOICE_PRESENCE_ENROLLMENT_TTL_SECONDS = 3600;

const VOICE_PRESENCE_HEARTBEAT_ACTIVE_PREFIX = 'voice:presence:v2:active:';
const VOICE_PRESENCE_HEARTBEAT_ENROLLED_PREFIX = 'voice:presence:v2:enrolled:';
const VOICE_PRESENCE_HEARTBEAT_CHANNEL_PREFIX = 'voice:presence:v2:channel:';

export class VoicePresenceHeartbeatStore {
	constructor(private readonly kvClient: IKVProvider) {}

	async recordHeartbeat(params: VoicePresenceHeartbeatParams): Promise<VoicePresenceHeartbeatResult> {
		const now = Date.now();
		const payload = JSON.stringify({
			version: 2,
			channelId: params.channelId.toString(),
			userId: params.userId.toString(),
			connectionId: params.connectionId,
			heartbeatAtMs: now,
		});
		await Promise.all([
			this.kvClient.setex(voicePresenceHeartbeatActiveKey(params), VOICE_PRESENCE_HEARTBEAT_TTL_SECONDS, payload),
			this.kvClient.setex(voicePresenceHeartbeatEnrollmentKey(params), VOICE_PRESENCE_ENROLLMENT_TTL_SECONDS, payload),
			this.kvClient.sadd(voicePresenceHeartbeatChannelKey(params.channelId), voicePresenceHeartbeatKeySuffix(params)),
			this.kvClient.expire(voicePresenceHeartbeatChannelKey(params.channelId), VOICE_PRESENCE_ENROLLMENT_TTL_SECONDS),
		]);
		return {
			heartbeatIntervalMs: VOICE_PRESENCE_HEARTBEAT_INTERVAL_MS,
			heartbeatTtlMs: VOICE_PRESENCE_HEARTBEAT_TTL_SECONDS * 1000,
			expiresAtMs: now + VOICE_PRESENCE_HEARTBEAT_TTL_SECONDS * 1000,
		};
	}

	async getHeartbeatState(params: VoicePresenceHeartbeatParams): Promise<VoicePresenceHeartbeatState> {
		const active = await this.kvClient.get(voicePresenceHeartbeatActiveKey(params));
		if (active !== null) {
			return 'active';
		}
		const enrolled = await this.kvClient.get(voicePresenceHeartbeatEnrollmentKey(params));
		return enrolled !== null ? 'expired' : 'legacy';
	}

	async markHeartbeatEnded(params: VoicePresenceHeartbeatParams): Promise<void> {
		const enrollmentKey = voicePresenceHeartbeatEnrollmentKey(params);
		const enrolled = await this.kvClient.get(enrollmentKey);
		const payload = JSON.stringify({
			version: 2,
			channelId: params.channelId.toString(),
			userId: params.userId.toString(),
			connectionId: params.connectionId,
			endedAtMs: Date.now(),
		});
		await Promise.all([
			this.kvClient.del(voicePresenceHeartbeatActiveKey(params)),
			this.kvClient.srem(voicePresenceHeartbeatChannelKey(params.channelId), voicePresenceHeartbeatKeySuffix(params)),
			enrolled === null
				? Promise.resolve()
				: this.kvClient.setex(enrollmentKey, VOICE_PRESENCE_ENROLLMENT_TTL_SECONDS, payload),
		]);
	}

	async markChannelHeartbeatsEnded(channelId: ChannelID): Promise<void> {
		const channelKey = voicePresenceHeartbeatChannelKey(channelId);
		const suffixes = await this.kvClient.smembers(channelKey);
		const activeKeys = suffixes.map((suffix) => `${VOICE_PRESENCE_HEARTBEAT_ACTIVE_PREFIX}${suffix}`);
		if (activeKeys.length > 0) {
			await this.kvClient.del(...activeKeys, channelKey);
			return;
		}
		await this.kvClient.del(channelKey);
	}
}

function voicePresenceHeartbeatActiveKey(params: VoicePresenceHeartbeatParams): string {
	return `${VOICE_PRESENCE_HEARTBEAT_ACTIVE_PREFIX}${voicePresenceHeartbeatKeySuffix(params)}`;
}

function voicePresenceHeartbeatEnrollmentKey(params: VoicePresenceHeartbeatParams): string {
	return `${VOICE_PRESENCE_HEARTBEAT_ENROLLED_PREFIX}${voicePresenceHeartbeatKeySuffix(params)}`;
}

function voicePresenceHeartbeatChannelKey(channelId: ChannelID): string {
	return `${VOICE_PRESENCE_HEARTBEAT_CHANNEL_PREFIX}${channelId.toString()}`;
}

function voicePresenceHeartbeatKeySuffix(params: VoicePresenceHeartbeatParams): string {
	const connectionHash = createHash('sha256').update(params.connectionId).digest('base64url');
	return `channel:${params.channelId.toString()}:user:${params.userId.toString()}:connection:${connectionHash}`;
}
