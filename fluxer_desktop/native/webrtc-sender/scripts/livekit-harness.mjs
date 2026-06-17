#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later

import {createHmac, randomUUID} from 'node:crypto';
import {writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import net from 'node:net';
import {setTimeout as delay} from 'node:timers/promises';
import {fileURLToPath} from 'node:url';

const require = createRequire(import.meta.url);
const {VoiceEngine, getHardwareEncoderCapabilities, isSupported, loadError} = require('../index.js');

const DEFAULT_URL = 'ws://localhost:7880';
const DEFAULT_API_KEY = 'devkey';
const DEFAULT_API_SECRET = 'secret';
const DEFAULT_TIMEOUT_MS = 30_000;
const DEFAULT_CONNECT_TIMEOUT_MS = 30_000;
const DEFAULT_DISCONNECT_TIMEOUT_MS = 10_000;
const DEFAULT_PROBE_TIMEOUT_MS = 1_500;
const DEFAULT_WIDTH = 320;
const DEFAULT_HEIGHT = 180;
const DEFAULT_FPS = 15;
const DEFAULT_SAMPLE_RATE = 48_000;
const DEFAULT_CHANNELS = 1;
const DEFAULT_AUDIO_CHUNK_MS = 20;
const STRICT_DURATION_MS = 10 * 60 * 1000;
const STRICT_MIN_FPS_RATIO = 0.95;
const STRICT_MAX_FRAME_GAP_MS = 250;
const STRICT_MAX_AV_DRIFT_MS = 80;
const STRICT_MAX_AUDIO_FRAME_GAP_MS = 250;
const AUDIO_POLL_INTERVAL_MS = 100;
const MIN_INBOUND_AUDIO_FRAMES = 3;
const MIN_INBOUND_VIDEO_FRAMES = 3;
const VIDEO_PATTERNS = new Set(['gradient', 'fast']);
const VIDEO_QUALITIES = new Set(['low', 'medium', 'high']);
const VIDEO_INPUTS = new Set(['bgra', 'nv12']);
const LIVEKIT_SCREEN_SHARE_SOURCE = 'screen_share';
const LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE = 'screen_share_audio';

function parseBooleanFlag(name, value) {
	const normalized = value.trim();
	if (/^(1|true|yes|on|required)$/i.test(normalized)) return true;
	if (/^(0|false|no|off|disabled)$/i.test(normalized)) return false;
	throw new Error(`${name} must be a boolean flag: 1/0, true/false, yes/no, or on/off`);
}

function envFlag(...names) {
	for (const name of names) {
		const value = process.env[name];
		if (value !== undefined && value.trim() !== '') return parseBooleanFlag(name, value);
	}
	return false;
}

function envOptionalFlag(name, fallback) {
	const value = process.env[name];
	if (value === undefined || value.trim() === '') return fallback;
	return parseBooleanFlag(name, value);
}

function envString(names, fallback) {
	for (const name of names) {
		const value = process.env[name];
		if (value?.trim()) return value.trim();
	}
	return fallback;
}

function envOptionalString(names) {
	for (const name of names) {
		const value = process.env[name];
		if (value?.trim()) return value.trim();
	}
	return null;
}

function envOptionalStringConsistent(names) {
	let selected = null;
	let selectedName = null;
	for (const name of names) {
		const value = process.env[name];
		if (!value?.trim()) continue;
		const trimmed = value.trim();
		if (selected !== null && trimmed !== selected) {
			throw new Error(`${name} conflicts with ${selectedName}; use one value for ${names.join(', ')}`);
		}
		selected = trimmed;
		selectedName = name;
	}
	return selected;
}

function envInteger(name, fallback) {
	const value = process.env[name];
	if (value === undefined || value === '') return fallback;
	const parsed = Number(value);
	if (!Number.isInteger(parsed) || parsed < 0) {
		throw new Error(`${name} must be a non-negative integer`);
	}
	return parsed;
}

function envPositiveInteger(name, fallback) {
	const parsed = envInteger(name, fallback);
	if (!Number.isInteger(parsed) || parsed <= 0) {
		throw new Error(`${name} must be a positive integer`);
	}
	return parsed;
}

function envNumber(name, fallback) {
	const value = process.env[name];
	if (value === undefined || value === '') return fallback;
	const parsed = Number(value);
	if (!Number.isFinite(parsed) || parsed < 0) {
		throw new Error(`${name} must be a non-negative number`);
	}
	return parsed;
}

function envPositiveNumber(name, fallback) {
	const value = process.env[name];
	if (value === undefined || value === '') return fallback;
	const parsed = Number(value);
	if (!Number.isFinite(parsed) || parsed <= 0) {
		throw new Error(`${name} must be a positive number`);
	}
	return parsed;
}

function envUnitNumber(name, fallback) {
	const parsed = envPositiveNumber(name, fallback);
	if (parsed > 1) {
		throw new Error(`${name} must be greater than 0 and less than or equal to 1`);
	}
	return parsed;
}

function envOptionalNonNegativeInteger(name) {
	const value = process.env[name];
	if (value === undefined || value === '') return null;
	const parsed = Number(value);
	if (!Number.isInteger(parsed) || parsed < 0) {
		throw new Error(`${name} must be a non-negative integer`);
	}
	return parsed;
}

function envVideoPattern(name, fallback) {
	const value = process.env[name]?.trim().toLowerCase() || fallback;
	if (!VIDEO_PATTERNS.has(value)) {
		throw new Error(`${name} must be one of: ${[...VIDEO_PATTERNS].join(', ')}`);
	}
	return value;
}

function envVideoInput(name, fallback) {
	const value = process.env[name]?.trim().toLowerCase() || fallback;
	if (!VIDEO_INPUTS.has(value)) {
		throw new Error(`${name} must be one of: ${[...VIDEO_INPUTS].join(', ')}`);
	}
	return value;
}

function envOptionalVideoQuality(name) {
	const value = process.env[name]?.trim().toLowerCase();
	if (!value) return null;
	if (!VIDEO_QUALITIES.has(value)) {
		throw new Error(`${name} must be one of: ${[...VIDEO_QUALITIES].join(', ')}`);
	}
	return value;
}

function jwtSubject(token) {
	const parts = token.split('.');
	if (parts.length < 2) {
		throw new Error('LiveKit token must be a JWT when identity is not supplied');
	}
	let payload;
	try {
		payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
	} catch (error) {
		throw new Error(`failed to decode LiveKit token payload: ${error.message}`);
	}
	if (typeof payload.sub !== 'string' || payload.sub.length === 0) {
		throw new Error('LiveKit token payload does not contain a subject identity');
	}
	return payload.sub;
}

function resolveIdentity({configuredIdentity, token, fallback}) {
	if (configuredIdentity) return configuredIdentity;
	if (token) return jwtSubject(token);
	return fallback;
}

function parseCodecList(rawValue, fallback) {
	const value = rawValue?.trim() ? rawValue : fallback;
	const codecs = value
		.split(',')
		.map((entry) => entry.trim())
		.filter((entry) => entry.length > 0);
	if (codecs.length === 0) {
		throw new Error('at least one LiveKit screen codec must be configured');
	}
	for (const codec of codecs) {
		expectedCodecMime(codec);
	}
	return codecs;
}

function parseConfig() {
	const strict = envFlag('LIVEKIT_HARNESS_STRICT', 'FLUXER_NATIVE_MEDIA_STRICT');
	const liveKitUrl = envString(['LIVEKIT_URL', 'LIVEKIT_WS_URL'], DEFAULT_URL);
	const generatedPublisherIdentity = `fluxer-native-publisher-${process.pid}`;
	const generatedSecondaryPublisherIdentity = `fluxer-native-publisher-2-${process.pid}`;
	const generatedSubscriberIdentity = `fluxer-native-subscriber-${process.pid}`;
	const publisherToken = envOptionalString(['LIVEKIT_PUBLISHER_TOKEN']);
	const secondaryPublisherToken = envOptionalString(['LIVEKIT_SECONDARY_PUBLISHER_TOKEN']);
	const subscriberToken = envOptionalString(['LIVEKIT_SUBSCRIBER_TOKEN']);
	const externalTokens = publisherToken !== null || secondaryPublisherToken !== null || subscriberToken !== null;
	const width = envPositiveInteger('LIVEKIT_SCREEN_WIDTH', DEFAULT_WIDTH);
	const height = envPositiveInteger('LIVEKIT_SCREEN_HEIGHT', DEFAULT_HEIGHT);
	if (width % 2 !== 0 || height % 2 !== 0) {
		throw new Error('LIVEKIT_SCREEN_WIDTH and LIVEKIT_SCREEN_HEIGHT must be even');
	}
	const fps = envPositiveNumber('LIVEKIT_SCREEN_FPS', DEFAULT_FPS);
	const minVideoFps = envPositiveNumber('LIVEKIT_MIN_VIDEO_FPS', 15);
	if (minVideoFps > fps) {
		throw new Error('LIVEKIT_MIN_VIDEO_FPS must be less than or equal to LIVEKIT_SCREEN_FPS');
	}
	const screenCodecs = parseCodecList(envOptionalString(['LIVEKIT_SCREEN_CODECS', 'LIVEKIT_SCREEN_CODEC']), 'vp8');
	const configuredExpectedScreenCodecs = envOptionalString([
		'LIVEKIT_EXPECT_SCREEN_CODECS',
		'LIVEKIT_EXPECT_SCREEN_CODEC',
	]);
	const expectedScreenCodecs = parseCodecList(configuredExpectedScreenCodecs, screenCodecs.join(','));
	if (configuredExpectedScreenCodecs !== null && expectedScreenCodecs.length !== screenCodecs.length) {
		throw new Error('LIVEKIT_EXPECT_SCREEN_CODECS length must match LIVEKIT_SCREEN_CODECS length');
	}
	const secondaryPublisherRequested = envOptionalFlag('LIVEKIT_ENABLE_SECOND_PUBLISHER', strict);
	const configuredSecondaryPublisherCodec = envOptionalStringConsistent([
		'LIVEKIT_SECOND_PUBLISHER_CODEC',
		'LIVEKIT_SECOND_PUBLISHER_SCREEN_CODEC',
		'LIVEKIT_SECOND_PUBLISHER_SCREEN_CODECS',
		'LIVEKIT_SECONDARY_PUBLISHER_SCREEN_CODEC',
		'LIVEKIT_SECONDARY_PUBLISHER_SCREEN_CODECS',
	]);
	const configuredExpectedSecondaryPublisherCodec = envOptionalStringConsistent([
		'LIVEKIT_EXPECT_SECOND_PUBLISHER_SCREEN_CODEC',
		'LIVEKIT_EXPECT_SECOND_PUBLISHER_SCREEN_CODECS',
		'LIVEKIT_EXPECT_SECONDARY_PUBLISHER_SCREEN_CODEC',
		'LIVEKIT_EXPECT_SECONDARY_PUBLISHER_SCREEN_CODECS',
	]);
	const secondaryPublisherCodec =
		configuredSecondaryPublisherCodec ?? (screenCodecs.length > 1 ? screenCodecs[1] : screenCodecs[0]);
	const expectedSecondaryPublisherCodec = configuredExpectedSecondaryPublisherCodec ?? secondaryPublisherCodec;
	const screenSimulcast = envOptionalFlag('LIVEKIT_SCREEN_SIMULCAST', true);
	const microphone = envOptionalFlag('LIVEKIT_ENABLE_MICROPHONE', true);
	const screenAudio = envOptionalFlag('LIVEKIT_ENABLE_SCREEN_AUDIO', strict);
	if (externalTokens) {
		if (!publisherToken || !subscriberToken) {
			throw new Error(
				'LIVEKIT_PUBLISHER_TOKEN and LIVEKIT_SUBSCRIBER_TOKEN are both required when using external tokens',
			);
		}
		if (secondaryPublisherRequested && !secondaryPublisherToken) {
			throw new Error('LIVEKIT_SECONDARY_PUBLISHER_TOKEN is required when LIVEKIT_ENABLE_SECOND_PUBLISHER=1');
		}
	}
	return {
		url: liveKitUrl,
		serverApiUrl: envString(['LIVEKIT_API_URL', 'LIVEKIT_HTTP_URL'], liveKitApiUrl(liveKitUrl)),
		apiKey: envString(['LIVEKIT_API_KEY'], DEFAULT_API_KEY),
		apiSecret: envString(['LIVEKIT_API_SECRET', 'LIVEKIT_SECRET'], DEFAULT_API_SECRET),
		room: envString(['LIVEKIT_ROOM'], `fluxer-webrtc-harness-${process.pid}-${randomUUID()}`),
		externalTokens,
		publisherToken,
		secondaryPublisherToken,
		subscriberToken,
		publisherIdentity: resolveIdentity({
			configuredIdentity: envOptionalString(['LIVEKIT_PUBLISHER_IDENTITY']),
			token: publisherToken,
			fallback: generatedPublisherIdentity,
		}),
		secondaryPublisherIdentity: resolveIdentity({
			configuredIdentity: envOptionalString(['LIVEKIT_SECONDARY_PUBLISHER_IDENTITY']),
			token: secondaryPublisherToken,
			fallback: generatedSecondaryPublisherIdentity,
		}),
		subscriberIdentity: resolveIdentity({
			configuredIdentity: envOptionalString(['LIVEKIT_SUBSCRIBER_IDENTITY']),
			token: subscriberToken,
			fallback: generatedSubscriberIdentity,
		}),
		required: envFlag('FLUXER_WEBRTC_SENDER_LIVEKIT_REQUIRED', 'LIVEKIT_REQUIRED'),
		strict,
		timeoutMs: envPositiveInteger('LIVEKIT_HARNESS_TIMEOUT_MS', DEFAULT_TIMEOUT_MS),
		connectTimeoutMs: envPositiveInteger('LIVEKIT_CONNECT_TIMEOUT_MS', DEFAULT_CONNECT_TIMEOUT_MS),
		disconnectTimeoutMs: envPositiveInteger('LIVEKIT_DISCONNECT_TIMEOUT_MS', DEFAULT_DISCONNECT_TIMEOUT_MS),
		probeTimeoutMs: envPositiveInteger('LIVEKIT_PROBE_TIMEOUT_MS', DEFAULT_PROBE_TIMEOUT_MS),
		durationMs: envInteger('LIVEKIT_HARNESS_DURATION_MS', strict ? STRICT_DURATION_MS : 0),
		width,
		height,
		expectedWidth: envPositiveInteger('LIVEKIT_EXPECT_SCREEN_WIDTH', width),
		expectedHeight: envPositiveInteger('LIVEKIT_EXPECT_SCREEN_HEIGHT', height),
		fps,
		minFpsRatio: envNumber('LIVEKIT_MIN_RECEIVED_FPS_RATIO', strict ? STRICT_MIN_FPS_RATIO : 0),
		maxFrameGapMs: envNumber('LIVEKIT_MAX_FRAME_GAP_MS', strict ? STRICT_MAX_FRAME_GAP_MS : 0),
		maxAudioFrameGapMs: envNumber('LIVEKIT_MAX_AUDIO_FRAME_GAP_MS', strict ? STRICT_MAX_AUDIO_FRAME_GAP_MS : 0),
		maxAvDriftMs: envNumber('LIVEKIT_MAX_AV_DRIFT_MS', strict ? STRICT_MAX_AV_DRIFT_MS : 0),
		maxPacketLoss: envOptionalNonNegativeInteger('LIVEKIT_MAX_PACKET_LOSS'),
		requireStableResolution: envOptionalFlag('LIVEKIT_REQUIRE_STABLE_RESOLUTION', strict),
		videoPattern: envVideoPattern('LIVEKIT_VIDEO_PATTERN', 'gradient'),
		videoInput: envVideoInput('LIVEKIT_VIDEO_INPUT', 'bgra'),
		subscriberVideoQuality: envOptionalVideoQuality('LIVEKIT_SUBSCRIBER_VIDEO_QUALITY'),
		screenCodecs,
		expectedScreenCodecs,
		codec: screenCodecs[0],
		expectedScreenCodec: expectedCodecMime(expectedScreenCodecs[0] ?? screenCodecs[0]),
		secondaryPublisher: secondaryPublisherRequested,
		secondaryPublisherCodec,
		secondaryPublisherCodecExplicit: configuredSecondaryPublisherCodec !== null,
		expectedSecondaryPublisherCodec: expectedCodecMime(expectedSecondaryPublisherCodec),
		expectedSecondaryPublisherCodecExplicit: configuredExpectedSecondaryPublisherCodec !== null,
		screenSimulcast,
		secondaryPublisherScreenSimulcast: envOptionalFlag('LIVEKIT_SECOND_PUBLISHER_SCREEN_SIMULCAST', screenSimulcast),
		adaptiveSend: envOptionalFlag('LIVEKIT_ADAPTIVE_SEND', true),
		minVideoFps,
		minResolutionScale: envUnitNumber('LIVEKIT_MIN_RESOLUTION_SCALE', 0.5),
		maxAudioBufferMs: envPositiveInteger('LIVEKIT_MAX_AUDIO_BUFFER_MS', 750),
		secondaryPublisherMicrophone: envOptionalFlag('LIVEKIT_SECOND_PUBLISHER_ENABLE_MICROPHONE', microphone),
		secondaryPublisherScreenAudio: envOptionalFlag('LIVEKIT_SECOND_PUBLISHER_ENABLE_SCREEN_AUDIO', screenAudio),
		maxBitrateBps: envNumber('LIVEKIT_SCREEN_MAX_BITRATE_BPS', 400_000),
		sampleRate: envPositiveInteger('LIVEKIT_AUDIO_SAMPLE_RATE', DEFAULT_SAMPLE_RATE),
		channels: envPositiveInteger('LIVEKIT_AUDIO_CHANNELS', DEFAULT_CHANNELS),
		audioChunkMs: envPositiveInteger('LIVEKIT_AUDIO_CHUNK_MS', DEFAULT_AUDIO_CHUNK_MS),
		microphone,
		screenAudio,
		dataPacket: envOptionalFlag('LIVEKIT_ENABLE_DATA_PACKET', strict),
		subscriptionCycle: envOptionalFlag('LIVEKIT_ENABLE_SUBSCRIPTION_CYCLE', strict),
		validateServerPublishing: envOptionalFlag('LIVEKIT_VALIDATE_SERVER_PUBLISHING', strict),
		camera: envOptionalFlag('LIVEKIT_ENABLE_CAMERA', false),
		cameraDeviceId: envOptionalString(['LIVEKIT_CAMERA_DEVICE_ID']),
		expectedCameraCodec: expectedCodecMime(envString(['LIVEKIT_EXPECT_CAMERA_CODEC'], '')),
		e2eeKey: envOptionalString(['LIVEKIT_E2EE_KEY']),
		expectedHardwareEncoder: envOptionalString(['LIVEKIT_EXPECT_HARDWARE_ENCODER']),
		reportPath: envOptionalString(['LIVEKIT_HARNESS_REPORT_PATH', 'FLUXER_NATIVE_MEDIA_REPORT']),
		verbose: envFlag('FLUXER_WEBRTC_SENDER_LIVEKIT_VERBOSE', 'LIVEKIT_VERBOSE'),
	};
}

function expectedCodecMime(codec) {
	switch (codec.trim().toLowerCase()) {
		case '':
			return null;
		case 'vp8':
			return 'video/VP8';
		case 'vp9':
			return 'video/VP9';
		case 'h264':
			return 'video/H264';
		case 'av1':
			return 'video/AV1';
		case 'h265':
		case 'hevc':
			return 'video/H265';
		default:
			throw new Error(`unsupported LIVEKIT_EXPECT_SCREEN_CODEC: ${codec}`);
	}
}

function liveKitTcpTarget(rawUrl) {
	let url;
	try {
		url = new URL(rawUrl);
	} catch (error) {
		throw new Error(`LIVEKIT_URL is not a valid URL: ${error.message}`);
	}
	if (url.protocol !== 'ws:' && url.protocol !== 'wss:') {
		throw new Error('LIVEKIT_URL must use ws:// or wss://');
	}
	return {
		host: url.hostname.replace(/^\[(.*)\]$/, '$1'),
		port: Number(url.port || (url.protocol === 'wss:' ? 443 : 80)),
	};
}

function liveKitApiUrl(rawUrl) {
	let url;
	try {
		url = new URL(rawUrl);
	} catch (error) {
		throw new Error(`LIVEKIT_URL is not a valid URL: ${error.message}`);
	}
	if (url.protocol === 'ws:') return `http://${url.host}`;
	if (url.protocol === 'wss:') return `https://${url.host}`;
	throw new Error('LIVEKIT_URL must use ws:// or wss://');
}

function probeTcp({host, port}, timeoutMs) {
	return new Promise((resolve) => {
		const socket = net.createConnection({host, port});
		let done = false;
		const finish = (result) => {
			if (done) return;
			done = true;
			socket.destroy();
			resolve(result);
		};
		socket.setTimeout(timeoutMs);
		socket.once('connect', () => finish({ok: true}));
		socket.once('timeout', () => finish({ok: false, reason: `timed out after ${timeoutMs}ms`}));
		socket.once('error', (error) => finish({ok: false, reason: error.message}));
	});
}

function base64urlJson(value) {
	return Buffer.from(JSON.stringify(value)).toString('base64url');
}

function createLiveKitToken({apiKey, apiSecret, room, identity, name}) {
	const now = Math.floor(Date.now() / 1000);
	const header = {alg: 'HS256', typ: 'JWT'};
	const payload = {
		iss: apiKey,
		sub: identity,
		name,
		nbf: now - 5,
		exp: now + 600,
		jti: randomUUID(),
		video: {
			roomJoin: true,
			room,
			canPublish: true,
			canSubscribe: true,
			canPublishData: true,
			canPublishSources: ['camera', 'microphone', 'screen_share', 'screen_share_audio'],
		},
	};
	const body = `${base64urlJson(header)}.${base64urlJson(payload)}`;
	const signature = createHmac('sha256', apiSecret).update(body).digest('base64url');
	return `${body}.${signature}`;
}

function createLiveKitAdminToken({apiKey, apiSecret, room}) {
	const now = Math.floor(Date.now() / 1000);
	const header = {alg: 'HS256', typ: 'JWT'};
	const payload = {
		iss: apiKey,
		sub: `fluxer-native-harness-admin-${process.pid}`,
		nbf: now - 5,
		exp: now + 600,
		jti: randomUUID(),
		video: {
			room,
			roomAdmin: true,
		},
	};
	const body = `${base64urlJson(header)}.${base64urlJson(payload)}`;
	const signature = createHmac('sha256', apiSecret).update(body).digest('base64url');
	return `${body}.${signature}`;
}

function parsePayload(jsonPayload) {
	try {
		return JSON.parse(jsonPayload);
	} catch {
		return {};
	}
}

function unpackNapiPair(args) {
	if (args.length === 1 && Array.isArray(args[0])) {
		return [args[0][0], args[0][1]];
	}
	return [args[0], args[1]];
}

function createEngineState(name, verbose) {
	return {
		name,
		events: [],
		stats: null,
		statsSamples: [],
		trackIndex: new Map(),
		videoCallbacks: 0,
		videoBytes: 0,
		videoFrameTimes: [],
		videoFrameRecords: [],
		maxVideoFrameGapMs: 0,
		lastVideoMeta: null,
		videoResolutionCounts: new Map(),
		lastVideoFrameAtMs: null,
		audioFrameCounts: [],
		audioFrameTimes: [],
		lastAudioFrameAtMs: null,
		maxAudioFrameGapMs: 0,
		recordEvent(eventType, jsonPayload) {
			const payload = parsePayload(jsonPayload);
			this.events.push({eventType, payload, atMs: Date.now()});
			if (typeof payload.trackSid === 'string' && payload.trackSid.length > 0) {
				if (eventType === 'trackSubscribed') {
					this.trackIndex.set(payload.trackSid, {
						identity: payload.identity,
						kind: normalizeTrackKind(payload.kind),
						source: normalizeTrackSource(payload.source),
						trackName: payload.trackName,
					});
				}
				if (eventType === 'trackUnsubscribed') {
					this.trackIndex.delete(payload.trackSid);
				}
			}
			if (eventType === 'stats') {
				this.stats = payload;
				this.statsSamples.push({payload, atMs: Date.now()});
			}
			if (verbose) {
				const suffix = eventType === 'stats' ? '' : ` ${JSON.stringify(payload)}`;
				console.log(`[livekit-harness] ${name} event: ${eventType}${suffix}`);
			}
		},
		recordVideoFrame(metaJson, data) {
			const now = Date.now();
			if (this.lastVideoFrameAtMs !== null) {
				this.maxVideoFrameGapMs = Math.max(this.maxVideoFrameGapMs, now - this.lastVideoFrameAtMs);
			}
			this.lastVideoFrameAtMs = now;
			this.videoFrameTimes.push(now);
			this.videoCallbacks += 1;
			this.videoBytes += data?.byteLength ?? data?.length ?? 0;
			const parsedMeta = parsePayload(metaJson);
			const indexedTrack =
				typeof parsedMeta.trackSid === 'string' ? this.trackIndex.get(parsedMeta.trackSid) : undefined;
			this.lastVideoMeta = {
				...parsedMeta,
				identity: parsedMeta.identity ?? indexedTrack?.identity,
				kind: parsedMeta.kind !== undefined ? normalizeTrackKind(parsedMeta.kind) : indexedTrack?.kind,
				source: parsedMeta.source !== undefined ? normalizeTrackSource(parsedMeta.source) : indexedTrack?.source,
				trackName: parsedMeta.trackName ?? indexedTrack?.trackName,
			};
			if (Number.isFinite(this.lastVideoMeta.width) && Number.isFinite(this.lastVideoMeta.height)) {
				const key = `${this.lastVideoMeta.width}x${this.lastVideoMeta.height}`;
				this.videoResolutionCounts.set(key, (this.videoResolutionCounts.get(key) ?? 0) + 1);
				this.videoFrameRecords.push({
					atMs: now,
					identity: this.lastVideoMeta.identity,
					kind: this.lastVideoMeta.kind,
					source: this.lastVideoMeta.source,
					trackName: this.lastVideoMeta.trackName,
					trackSid: this.lastVideoMeta.trackSid,
					width: this.lastVideoMeta.width,
					height: this.lastVideoMeta.height,
				});
			}
			if (verbose && this.videoCallbacks === 1) {
				console.log(`[livekit-harness] ${name} first video frame: ${metaJson}`);
			}
		},
		recordAudioFrameCount(count) {
			const now = Date.now();
			const last = this.audioFrameCounts[this.audioFrameCounts.length - 1];
			if (last && count > last.count) {
				if (this.lastAudioFrameAtMs !== null) {
					this.maxAudioFrameGapMs = Math.max(this.maxAudioFrameGapMs, now - this.lastAudioFrameAtMs);
				}
				this.lastAudioFrameAtMs = now;
				this.audioFrameTimes.push(now);
			}
			this.audioFrameCounts.push({count, atMs: now});
		},
	};
}

function hasEvent(state, eventType, predicate = () => true) {
	return state.events.some((event) => event.eventType === eventType && predicate(event.payload));
}

function statsHasOutbound(stats, kind, source) {
	return (
		Array.isArray(stats?.outbound) &&
		stats.outbound.some((entry) => entry.kind === kind && normalizeTrackSource(entry.source) === source)
	);
}

function statsHasInbound(stats, kind) {
	return Array.isArray(stats?.inbound) && stats.inbound.some((entry) => entry.kind === kind);
}

function statsInboundCount(stats, kind) {
	return Array.isArray(stats?.inbound) ? stats.inbound.filter((entry) => entry.kind === kind).length : 0;
}

function totalPacketsLost(stats) {
	const outboundLoss = Array.isArray(stats?.outbound)
		? stats.outbound.reduce((sum, entry) => sum + Math.max(0, Number(entry.packetsLost) || 0), 0)
		: 0;
	const inboundLoss = Array.isArray(stats?.inbound)
		? stats.inbound.reduce((sum, entry) => sum + Math.max(0, Number(entry.packetsLost) || 0), 0)
		: 0;
	return outboundLoss + inboundLoss;
}

function maxObservedPacketsLost(...states) {
	let max = 0;
	for (const state of states) {
		for (const sample of state?.statsSamples ?? []) {
			max = Math.max(max, totalPacketsLost(sample.payload));
		}
		max = Math.max(max, totalPacketsLost(state?.stats));
	}
	return max;
}

function maxObservedPacketsLostDelta(startedAtMs, endedAtMs, ...states) {
	let maxDelta = 0;
	for (const state of states) {
		const samples = [...(state?.statsSamples ?? [])];
		if (state?.stats) samples.push({payload: state.stats, atMs: endedAtMs});
		samples.sort((a, b) => a.atMs - b.atMs);
		let baseline = 0;
		let maxDuring = 0;
		let hasWindowSample = false;
		for (const sample of samples) {
			const total = totalPacketsLost(sample.payload);
			if (sample.atMs <= startedAtMs) {
				baseline = total;
				continue;
			}
			if (sample.atMs <= endedAtMs) {
				hasWindowSample = true;
				maxDuring = Math.max(maxDuring, total);
			}
		}
		if (hasWindowSample) maxDelta = Math.max(maxDelta, Math.max(0, maxDuring - baseline));
	}
	return maxDelta;
}

function statsHasOutboundCodec(stats, kind, source, expectedCodec) {
	return (
		Array.isArray(stats?.outbound) &&
		stats.outbound.some(
			(entry) => entry.kind === kind && normalizeTrackSource(entry.source) === source && entry.codec === expectedCodec,
		)
	);
}

function hasTrackEvent(state, eventType, {identity, kind, source}) {
	if (!state) return false;
	return hasEvent(
		state,
		eventType,
		(payload) =>
			(identity === undefined || payload.identity === identity) &&
			(kind === undefined || payload.kind === kind) &&
			(source === undefined || normalizeTrackSource(payload.source) === source),
	);
}

function statsHasInboundCodec(stats, kind, expectedCodec) {
	return (
		Array.isArray(stats?.inbound) && stats.inbound.some((entry) => entry.kind === kind && entry.codec === expectedCodec)
	);
}

function startVideoPump(_engine, config, _pumpErrors) {
	const stats = {
		input: config.videoInput,
		pattern: config.videoPattern,
		targetFps: config.fps,
		startedAtMs: Date.now(),
		stoppedAtMs: null,
		framesPushed: 0,
		framesRejected: 0,
		errors: 0,
		maxPushGapMs: 0,
		lastPushAtMs: null,
	};
	let stopped = false;
	return {
		stats() {
			const endedAtMs = stats.stoppedAtMs ?? Date.now();
			const elapsedMs = Math.max(1, endedAtMs - stats.startedAtMs);
			return {
				...stats,
				elapsedMs,
				producedFps: Math.round((stats.framesPushed / (elapsedMs / 1000)) * 100) / 100,
			};
		},
		stop() {
			if (stopped) return;
			stopped = true;
			stats.stoppedAtMs = Date.now();
		},
	};
}

function syntheticPcmFrame({sampleRate, channels, chunkMs, cursor}) {
	const samplesPerChannel = Math.round((sampleRate * chunkMs) / 1000);
	const buffer = Buffer.alloc(samplesPerChannel * channels * 2);
	const frequencyHz = 440;
	for (let i = 0; i < samplesPerChannel; i += 1) {
		const sample = Math.round(Math.sin(((cursor + i) / sampleRate) * frequencyHz * Math.PI * 2) * 10_000);
		for (let channel = 0; channel < channels; channel += 1) {
			buffer.writeInt16LE(sample, (i * channels + channel) * 2);
		}
	}
	return {buffer, nextCursor: cursor + samplesPerChannel};
}

function startAudioPump(label, pushFrame, config, pumpErrors) {
	let stopped = false;
	let cursor = 0;
	const promise = (async () => {
		while (!stopped) {
			const frame = syntheticPcmFrame({
				sampleRate: config.sampleRate,
				channels: config.channels,
				chunkMs: config.audioChunkMs,
				cursor,
			});
			cursor = frame.nextCursor;
			try {
				const ok = await pushFrame(frame.buffer, config.sampleRate, config.channels);
				if (!ok) pumpErrors.push(new Error(`${label} returned false`));
			} catch (error) {
				pumpErrors.push(error);
			}
			await delay(config.audioChunkMs);
		}
	})();
	return {
		stop() {
			stopped = true;
		},
		async done() {
			await promise;
		},
	};
}

function startAudioSampler(engine, state) {
	const timer = setInterval(() => {
		try {
			state.recordAudioFrameCount(engine.inboundAudioFrames());
		} catch {}
	}, AUDIO_POLL_INTERVAL_MS);
	timer.unref?.();
	return {
		stop() {
			clearInterval(timer);
		},
	};
}

function receivedFpsFromTimes(frames) {
	if (frames.length < 2) return 0;
	const elapsedSec = Math.max(0.001, (frames[frames.length - 1] - frames[0]) / 1000);
	return (frames.length - 1) / elapsedSec;
}

function maxGapMs(times, startedAtMs, endedAtMs) {
	const windowTimes = times.filter((time) => time >= startedAtMs && time <= endedAtMs);
	let maxGap = 0;
	for (let index = 1; index < windowTimes.length; index += 1) {
		maxGap = Math.max(maxGap, windowTimes[index] - windowTimes[index - 1]);
	}
	return maxGap;
}

function countWindowSamples(times, startedAtMs, endedAtMs) {
	return times.filter((time) => time >= startedAtMs && time <= endedAtMs).length;
}

function primaryScreenVideoSelector(config) {
	return {
		identity: config.publisherIdentity,
		kind: 'video',
		source: LIVEKIT_SCREEN_SHARE_SOURCE,
	};
}

function frameRecordMatches(record, selector) {
	if (!record) return false;
	if (selector.kind !== undefined && record.kind !== undefined && record.kind !== selector.kind) return false;
	if (selector.source !== undefined && record.source !== undefined && record.source !== selector.source) return false;
	if (selector.identity !== undefined && record.identity !== undefined && record.identity !== selector.identity) {
		return false;
	}
	return true;
}

function selectedVideoFrameRecords(state, startedAtMs, endedAtMs, selector) {
	const records = [];
	for (const record of state?.videoFrameRecords ?? []) {
		if (record.atMs < startedAtMs || record.atMs > endedAtMs) continue;
		if (!frameRecordMatches(record, selector)) continue;
		records.push(record);
	}
	return records;
}

function selectedVideoFrameTimes(state, startedAtMs, endedAtMs, selector) {
	return selectedVideoFrameRecords(state, startedAtMs, endedAtMs, selector).map((record) => record.atMs);
}

function lastVideoFrameRecord(state, startedAtMs, endedAtMs, selector) {
	return selectedVideoFrameRecords(state, startedAtMs, endedAtMs, selector).at(-1) ?? null;
}

function videoResolutionCounts(state, startedAtMs, endedAtMs, selector) {
	const counts = new Map();
	for (const record of selectedVideoFrameRecords(state, startedAtMs, endedAtMs, selector)) {
		const key = `${record.width}x${record.height}`;
		counts.set(key, (counts.get(key) ?? 0) + 1);
	}
	return Object.fromEntries(counts);
}

function videoResolutionMismatchCount(state, expectedWidth, expectedHeight, startedAtMs, endedAtMs, selector) {
	let mismatches = 0;
	for (const record of selectedVideoFrameRecords(state, startedAtMs, endedAtMs, selector)) {
		if (record.width !== expectedWidth || record.height !== expectedHeight) {
			mismatches += 1;
		}
	}
	return mismatches;
}

function currentAvDriftMs(state, startedAtMs, endedAtMs, selector) {
	const videoTimes = selectedVideoFrameTimes(state, startedAtMs, endedAtMs, selector);
	const audioTimes = (state?.audioFrameTimes ?? []).filter((time) => time >= startedAtMs && time <= endedAtMs);
	const lastVideoFrameAtMs = videoTimes.at(-1) ?? null;
	const lastAudioFrameAtMs = audioTimes.at(-1) ?? null;
	if (lastVideoFrameAtMs === null || lastAudioFrameAtMs === null) return null;
	return Math.abs(lastVideoFrameAtMs - lastAudioFrameAtMs);
}

function expectedInboundVideoTrackCount(config) {
	let count = 1;
	if (config.camera) count += 1;
	if (config.secondaryPublisher) count += 1;
	return count;
}

function expectedInboundAudioTrackCount(config) {
	let count = 0;
	if (config.microphone) count += 1;
	if (config.screenAudio) count += 1;
	if (config.secondaryPublisher && config.secondaryPublisherMicrophone) count += 1;
	if (config.secondaryPublisher && config.secondaryPublisherScreenAudio) count += 1;
	return count;
}

function normalizeTrackKind(kind) {
	switch (String(kind ?? '').toLowerCase()) {
		case '0':
		case 'audio':
			return 'audio';
		case '1':
		case 'video':
			return 'video';
		case '2':
		case 'data':
			return 'data';
		default:
			return String(kind ?? '').toLowerCase();
	}
}

function normalizeTrackSource(source) {
	switch (String(source ?? '').toLowerCase()) {
		case '1':
		case 'camera':
		case 'camera_source':
			return 'camera';
		case '2':
		case 'microphone':
		case 'mic':
			return 'microphone';
		case '3':
		case 'screen_share':
		case 'screenshare':
		case 'screen-share':
			return LIVEKIT_SCREEN_SHARE_SOURCE;
		case '4':
		case 'screen_share_audio':
		case 'screenshareaudio':
		case 'screen-share-audio':
			return LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE;
		default:
			return String(source ?? '');
	}
}

function findServerParticipant(serverState, identity) {
	return serverState?.participants?.find((participant) => participant.identity === identity) ?? null;
}

function normalizeMimeType(mimeType) {
	if (typeof mimeType !== 'string' || mimeType.trim() === '') return null;
	return mimeType.trim().toLowerCase();
}

function serverTrackMatches(track, {kind, source, mimeType}) {
	if (normalizeTrackKind(track.type ?? track.kind) !== kind) return false;
	if (normalizeTrackSource(track.source) !== source) return false;
	if (track.muted !== false) return false;
	const expectedMimeType = normalizeMimeType(mimeType);
	if (expectedMimeType !== null && normalizeMimeType(track.mimeType ?? track.mime_type) !== expectedMimeType) {
		return false;
	}
	return true;
}

function serverParticipantHasTrack(serverState, identity, expectedTrack) {
	const participant = findServerParticipant(serverState, identity);
	if (!participant) return false;
	const tracks = participant.tracks ?? [];
	return tracks.some((track) => serverTrackMatches(track, expectedTrack));
}

function expectedServerTracks(config, publisherIdentity, secondaryPublisherIdentity) {
	const tracks = [
		{
			identity: publisherIdentity,
			kind: 'video',
			source: LIVEKIT_SCREEN_SHARE_SOURCE,
			mimeType: config.expectedScreenCodec,
			label: 'publisher screenshare',
		},
	];
	if (config.microphone) {
		tracks.push({identity: publisherIdentity, kind: 'audio', source: 'microphone', label: 'publisher microphone'});
	}
	if (config.screenAudio) {
		tracks.push({
			identity: publisherIdentity,
			kind: 'audio',
			source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
			label: 'publisher screen-share audio',
		});
	}
	if (config.camera) {
		tracks.push({
			identity: publisherIdentity,
			kind: 'video',
			source: 'camera',
			mimeType: config.expectedCameraCodec,
			label: 'publisher camera',
		});
	}
	if (config.secondaryPublisher) {
		tracks.push({
			identity: secondaryPublisherIdentity,
			kind: 'video',
			source: LIVEKIT_SCREEN_SHARE_SOURCE,
			mimeType: config.expectedSecondaryPublisherCodec,
			label: 'secondary publisher screenshare',
		});
		if (config.secondaryPublisherMicrophone) {
			tracks.push({
				identity: secondaryPublisherIdentity,
				kind: 'audio',
				source: 'microphone',
				label: 'secondary publisher microphone',
			});
		}
		if (config.secondaryPublisherScreenAudio) {
			tracks.push({
				identity: secondaryPublisherIdentity,
				kind: 'audio',
				source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
				label: 'secondary publisher screen-share audio',
			});
		}
	}
	return tracks;
}

function serverPublishedTrackChecks({config, serverState, publisherIdentity, secondaryPublisherIdentity}) {
	if (!config.validateServerPublishing) return [];
	const checks = [
		{
			name: 'server publishing API participants listed',
			pass: Array.isArray(serverState?.participants) && !serverState?.error,
		},
	];
	for (const track of expectedServerTracks(config, publisherIdentity, secondaryPublisherIdentity)) {
		checks.push({
			name: `server sees ${track.label} publication`,
			pass: serverParticipantHasTrack(serverState, track.identity, {
				kind: track.kind,
				source: track.source,
				mimeType: track.mimeType,
			}),
		});
	}
	return checks;
}

function currentChecks({
	publisher,
	subscriber,
	publisherState,
	subscriberState,
	config,
	publisherIdentity,
	secondaryPublisher,
	secondaryPublisherState,
	secondaryPublisherIdentity,
	serverState,
}) {
	const expectedVideoTracks = expectedInboundVideoTrackCount(config);
	const expectedAudioTracks = expectedInboundAudioTrackCount(config);
	const screenVideoMeta = lastVideoFrameRecord(
		subscriberState,
		Number.NEGATIVE_INFINITY,
		Number.POSITIVE_INFINITY,
		primaryScreenVideoSelector(config),
	);
	const checks = [
		{name: 'publisher connected', pass: publisher.isConnected()},
		{name: 'subscriber connected', pass: subscriber.isConnected()},
		{
			name: 'publisher local screenshare publication',
			pass: hasTrackEvent(publisherState, 'localTrackPublished', {
				kind: 'video',
				source: LIVEKIT_SCREEN_SHARE_SOURCE,
			}),
		},
		{
			name: 'subscriber remote screenshare subscription',
			pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
				identity: publisherIdentity,
				kind: 'video',
				source: LIVEKIT_SCREEN_SHARE_SOURCE,
			}),
		},
		{
			name: `subscriber inbound video frames >= ${MIN_INBOUND_VIDEO_FRAMES * expectedVideoTracks}`,
			pass: subscriber.inboundVideoFrames() >= MIN_INBOUND_VIDEO_FRAMES * expectedVideoTracks,
		},
		{
			name: 'subscriber video frame callback',
			pass: subscriberState.videoCallbacks > 0 && subscriberState.videoBytes > 0,
		},
		{
			name: 'publisher outbound screenshare stats',
			pass: statsHasOutbound(publisherState.stats, 'video', LIVEKIT_SCREEN_SHARE_SOURCE),
		},
		{
			name: 'subscriber inbound video stats',
			pass: statsHasInbound(subscriberState.stats, 'video'),
		},
		{
			name: `subscriber inbound video stats count >= ${expectedVideoTracks}`,
			pass: statsInboundCount(subscriberState.stats, 'video') >= expectedVideoTracks,
		},
		{
			name: `subscriber video resolution ${config.expectedWidth}x${config.expectedHeight}`,
			pass: screenVideoMeta?.width === config.expectedWidth && screenVideoMeta?.height === config.expectedHeight,
		},
	];
	if (expectedAudioTracks > 0) {
		checks.push(
			{
				name: `subscriber aggregate inbound audio frames >= ${MIN_INBOUND_AUDIO_FRAMES * expectedAudioTracks}`,
				pass: subscriber.inboundAudioFrames() >= MIN_INBOUND_AUDIO_FRAMES * expectedAudioTracks,
			},
			{
				name: `subscriber inbound audio stats count >= ${expectedAudioTracks}`,
				pass: statsInboundCount(subscriberState.stats, 'audio') >= expectedAudioTracks,
			},
		);
	}
	if (config.microphone) {
		checks.push(
			{
				name: 'publisher local microphone publication',
				pass: hasTrackEvent(publisherState, 'localTrackPublished', {kind: 'audio', source: 'microphone'}),
			},
			{
				name: 'subscriber remote microphone subscription',
				pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
					identity: publisherIdentity,
					kind: 'audio',
					source: 'microphone',
				}),
			},
			{
				name: `subscriber inbound audio frames >= ${MIN_INBOUND_AUDIO_FRAMES * Math.max(1, expectedAudioTracks)}`,
				pass: subscriber.inboundAudioFrames() >= MIN_INBOUND_AUDIO_FRAMES * Math.max(1, expectedAudioTracks),
			},
			{
				name: 'publisher outbound microphone stats',
				pass: statsHasOutbound(publisherState.stats, 'audio', 'microphone'),
			},
			{
				name: 'subscriber inbound audio stats',
				pass: statsHasInbound(subscriberState.stats, 'audio'),
			},
		);
	}
	if (config.screenAudio) {
		checks.push(
			{
				name: 'publisher local screen-share audio publication',
				pass: hasTrackEvent(publisherState, 'localTrackPublished', {
					kind: 'audio',
					source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
				}),
			},
			{
				name: 'subscriber remote screen-share audio subscription',
				pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
					identity: publisherIdentity,
					kind: 'audio',
					source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
				}),
			},
			{
				name: 'publisher outbound screen-share audio stats',
				pass: statsHasOutbound(publisherState.stats, 'audio', LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE),
			},
		);
	}
	if (config.camera) {
		checks.push(
			{
				name: 'publisher local camera publication',
				pass: hasTrackEvent(publisherState, 'localTrackPublished', {kind: 'video', source: 'camera'}),
			},
			{
				name: 'subscriber remote camera subscription',
				pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
					identity: publisherIdentity,
					kind: 'video',
					source: 'camera',
				}),
			},
			{
				name: 'publisher outbound camera stats',
				pass: statsHasOutbound(publisherState.stats, 'video', 'camera'),
			},
		);
		if (config.expectedCameraCodec) {
			checks.push({
				name: `publisher outbound camera codec ${config.expectedCameraCodec}`,
				pass: statsHasOutboundCodec(publisherState.stats, 'video', 'camera', config.expectedCameraCodec),
			});
		}
	}
	if (config.secondaryPublisher) {
		checks.push(
			{name: 'secondary publisher connected', pass: secondaryPublisher?.isConnected() === true},
			{
				name: 'secondary publisher local screenshare publication',
				pass: hasTrackEvent(secondaryPublisherState, 'localTrackPublished', {
					kind: 'video',
					source: LIVEKIT_SCREEN_SHARE_SOURCE,
				}),
			},
			{
				name: 'subscriber remote secondary screenshare subscription',
				pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
					identity: secondaryPublisherIdentity,
					kind: 'video',
					source: LIVEKIT_SCREEN_SHARE_SOURCE,
				}),
			},
			{
				name: 'secondary publisher outbound screenshare stats',
				pass: statsHasOutbound(secondaryPublisherState?.stats, 'video', LIVEKIT_SCREEN_SHARE_SOURCE),
			},
		);
		if (config.expectedSecondaryPublisherCodec) {
			checks.push(
				{
					name: `secondary publisher outbound screenshare codec ${config.expectedSecondaryPublisherCodec}`,
					pass: statsHasOutboundCodec(
						secondaryPublisherState?.stats,
						'video',
						LIVEKIT_SCREEN_SHARE_SOURCE,
						config.expectedSecondaryPublisherCodec,
					),
				},
				{
					name: `subscriber inbound secondary codec ${config.expectedSecondaryPublisherCodec}`,
					pass: statsHasInboundCodec(subscriberState.stats, 'video', config.expectedSecondaryPublisherCodec),
				},
			);
		}
		if (config.secondaryPublisherMicrophone) {
			checks.push(
				{
					name: 'secondary publisher local microphone publication',
					pass: hasTrackEvent(secondaryPublisherState, 'localTrackPublished', {
						kind: 'audio',
						source: 'microphone',
					}),
				},
				{
					name: 'subscriber remote secondary microphone subscription',
					pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
						identity: secondaryPublisherIdentity,
						kind: 'audio',
						source: 'microphone',
					}),
				},
				{
					name: 'secondary publisher outbound microphone stats',
					pass: statsHasOutbound(secondaryPublisherState?.stats, 'audio', 'microphone'),
				},
			);
		}
		if (config.secondaryPublisherScreenAudio) {
			checks.push(
				{
					name: 'secondary publisher local screen-share audio publication',
					pass: hasTrackEvent(secondaryPublisherState, 'localTrackPublished', {
						kind: 'audio',
						source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
					}),
				},
				{
					name: 'subscriber remote secondary screen-share audio subscription',
					pass: hasTrackEvent(subscriberState, 'trackSubscribed', {
						identity: secondaryPublisherIdentity,
						kind: 'audio',
						source: LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE,
					}),
				},
				{
					name: 'secondary publisher outbound screen-share audio stats',
					pass: statsHasOutbound(secondaryPublisherState?.stats, 'audio', LIVEKIT_SCREEN_SHARE_AUDIO_SOURCE),
				},
			);
		}
	}
	if (config.dataPacket) {
		checks.push({
			name: 'subscriber data packet received',
			pass: hasEvent(subscriberState, 'dataReceived', (payload) => payload.topic === 'native-media-harness'),
		});
		if (config.secondaryPublisher) {
			checks.push({
				name: 'subscriber secondary data packet received',
				pass: hasEvent(
					subscriberState,
					'dataReceived',
					(payload) => payload.topic === 'native-media-harness-secondary',
				),
			});
		}
	}
	if (config.expectedScreenCodec) {
		checks.push(
			{
				name: `publisher outbound screenshare codec ${config.expectedScreenCodec}`,
				pass: statsHasOutboundCodec(
					publisherState.stats,
					'video',
					LIVEKIT_SCREEN_SHARE_SOURCE,
					config.expectedScreenCodec,
				),
			},
			{
				name: `subscriber inbound video codec ${config.expectedScreenCodec}`,
				pass: statsHasInboundCodec(subscriberState.stats, 'video', config.expectedScreenCodec),
			},
		);
	}
	if (config.expectedHardwareEncoder) {
		const capabilities = getHardwareEncoderCapabilities();
		checks.push({
			name: `hardware encoder backend ${config.expectedHardwareEncoder}`,
			pass:
				capabilities.available === true &&
				String(capabilities.backend).toLowerCase() === config.expectedHardwareEncoder.toLowerCase(),
		});
	}
	checks.push(...serverPublishedTrackChecks({config, serverState, publisherIdentity, secondaryPublisherIdentity}));
	return checks;
}

async function waitForHarnessSuccess(context, pumpErrors, timeoutMs) {
	const start = Date.now();
	let checks = currentChecks(context);
	while (Date.now() - start < timeoutMs) {
		if (pumpErrors.length > 0) {
			throw pumpErrors[0];
		}
		await enforceSubscriberVideoQuality(context);
		await refreshServerPublishingState(context);
		checks = currentChecks(context);
		if (checks.every((check) => check.pass)) return checks;
		await delay(250);
	}
	const missing = checks.filter((check) => !check.pass).map((check) => check.name);
	throw new Error(`timed out after ${timeoutMs}ms waiting for: ${missing.join(', ')}`);
}

async function holdStrictDuration(context, pumpErrors, durationMs) {
	const startedAtMs = Date.now();
	context.strictStartedAtMs = startedAtMs;
	if (durationMs <= 0) {
		context.strictEndedAtMs = startedAtMs;
		return;
	}
	const endAtMs = startedAtMs + durationMs;
	while (Date.now() < endAtMs) {
		if (pumpErrors.length > 0) throw pumpErrors[0];
		await delay(1000);
	}
	context.strictEndedAtMs = Date.now();
}

async function withTimeout(promise, timeoutMs, label) {
	const timeout = delay(timeoutMs).then(() => {
		throw new Error(`${label} timed out after ${timeoutMs}ms`);
	});
	return Promise.race([promise, timeout]);
}

async function disconnectEngine(engine, label, timeoutMs) {
	if (!engine) return;
	try {
		await withTimeout(engine.disconnect(), timeoutMs, `${label} disconnect`);
	} catch (error) {
		console.warn(`[livekit-harness] ${label} disconnect failed: ${error.message}`);
	}
}

async function refreshServerPublishingState(context) {
	const {config, serverState} = context;
	if (!config.validateServerPublishing) return;
	const now = Date.now();
	if (serverState.inFlight || now - serverState.lastAttemptAtMs < 500) return;
	serverState.inFlight = true;
	serverState.lastAttemptAtMs = now;
	try {
		serverState.participants = await listServerParticipants(config);
		serverState.error = null;
		serverState.fetchedAtMs = Date.now();
	} catch (error) {
		serverState.error = String(error.message ?? error);
	} finally {
		serverState.inFlight = false;
	}
}

async function listServerParticipants(config) {
	const token = createLiveKitAdminToken({
		apiKey: config.apiKey,
		apiSecret: config.apiSecret,
		room: config.room,
	});
	const response = await fetch(`${config.serverApiUrl}/twirp/livekit.RoomService/ListParticipants`, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${token}`,
			'Content-Type': 'application/json',
		},
		body: JSON.stringify({room: config.room}),
	});
	const bodyText = await response.text();
	if (!response.ok) {
		throw new Error(`RoomService.ListParticipants failed with HTTP ${response.status}: ${bodyText.slice(0, 240)}`);
	}
	let body;
	try {
		body = bodyText ? JSON.parse(bodyText) : {};
	} catch (error) {
		throw new Error(`RoomService.ListParticipants returned invalid JSON: ${error.message}`);
	}
	if (!Array.isArray(body.participants)) {
		throw new Error('RoomService.ListParticipants response did not contain participants');
	}
	return body.participants;
}

async function runSubscriptionCycle(subscriber, publisherIdentity) {
	await subscriber.setRemoteTrackSubscription({
		participantIdentity: publisherIdentity,
		source: LIVEKIT_SCREEN_SHARE_SOURCE,
		subscribed: true,
		enabled: false,
		quality: 'low',
	});
	await delay(500);
	await subscriber.setRemoteTrackSubscription({
		participantIdentity: publisherIdentity,
		source: LIVEKIT_SCREEN_SHARE_SOURCE,
		subscribed: true,
		enabled: true,
		quality: 'high',
	});
}

async function setSubscriberVideoQuality(subscriber, publisherIdentity, quality) {
	if (!quality) return;
	await subscriber.setRemoteTrackSubscription({
		participantIdentity: publisherIdentity,
		source: LIVEKIT_SCREEN_SHARE_SOURCE,
		subscribed: true,
		enabled: true,
		quality,
	});
}

async function enforceSubscriberVideoQuality(context) {
	await setSubscriberVideoQuality(context.subscriber, context.publisherIdentity, context.config.subscriberVideoQuality);
	if (context.secondaryPublisher) {
		await setSubscriberVideoQuality(
			context.subscriber,
			context.secondaryPublisherIdentity,
			context.config.subscriberVideoQuality,
		);
	}
}

function sanitizeConfig(config) {
	const {
		apiSecret: _apiSecret,
		e2eeKey: _e2eeKey,
		publisherToken,
		secondaryPublisherToken,
		subscriberToken,
		...safe
	} = config;
	return {
		...safe,
		url: sanitizeReportUrl(config.url),
		serverApiUrl: sanitizeReportUrl(config.serverApiUrl),
		apiSecret: '<redacted>',
		e2eeKey: config.e2eeKey ? '<present>' : null,
		publisherToken: publisherToken ? '<present>' : null,
		secondaryPublisherToken: secondaryPublisherToken ? '<present>' : null,
		subscriberToken: subscriberToken ? '<present>' : null,
	};
}

function summarizeServerPublishing(serverState) {
	if (!serverState) return null;
	return {
		fetchedAtMs: serverState.fetchedAtMs,
		error: serverState.error,
		participants: (serverState.participants ?? []).map((participant) => ({
			identity: participant.identity,
			trackCount: participant.tracks?.length ?? 0,
			tracks: (participant.tracks ?? []).map((track) => ({
				sid: track.sid ?? null,
				name: track.name ?? null,
				kind: normalizeTrackKind(track.type ?? track.kind),
				source: normalizeTrackSource(track.source),
				mimeType: track.mimeType ?? track.mime_type ?? null,
				muted: track.muted ?? null,
			})),
		})),
	};
}

function sanitizeReportUrl(rawUrl) {
	if (typeof rawUrl !== 'string') return rawUrl;
	try {
		const url = new URL(rawUrl);
		if (!url.username && !url.password && !url.search && !url.hash) return rawUrl;
		url.username = '';
		url.password = '';
		url.search = '';
		url.hash = '';
		return url.toString();
	} catch {
		return rawUrl.replace(/\/\/[^/@\s]+@/g, '//<redacted>@').replace(/[?][^\s#]*/g, '?<redacted>');
	}
}

function scenarioRoomName(room, scenarioName, scenarioCount, externalTokens) {
	if (scenarioCount <= 1 || externalTokens) return room;
	const suffix = scenarioName.replace(/[^a-z0-9_.-]/gi, '-').replace(/-+/g, '-');
	return `${room}-${suffix}`.slice(0, 128);
}

function buildScenarioConfigs(baseConfig) {
	const scenarioCount = baseConfig.screenCodecs.length;
	return baseConfig.screenCodecs.map((codec, index) => {
		const expectedCodec = baseConfig.expectedScreenCodecs[index] ?? codec;
		const secondaryCodec = baseConfig.secondaryPublisherCodecExplicit
			? baseConfig.secondaryPublisherCodec
			: (baseConfig.screenCodecs[(index + 1) % baseConfig.screenCodecs.length] ?? codec);
		const expectedSecondaryCodec = baseConfig.expectedSecondaryPublisherCodecExplicit
			? baseConfig.expectedSecondaryPublisherCodec
			: expectedCodecMime(secondaryCodec);
		const scenarioName = baseConfig.secondaryPublisher ? `codec-${codec}-with-${secondaryCodec}` : `codec-${codec}`;
		return {
			...baseConfig,
			scenarioName,
			room: scenarioRoomName(baseConfig.room, scenarioName, scenarioCount, baseConfig.externalTokens),
			codec,
			expectedScreenCodec: expectedCodecMime(expectedCodec),
			secondaryPublisherCodec: secondaryCodec,
			expectedSecondaryPublisherCodec: expectedSecondaryCodec,
			reportPath: scenarioCount > 1 ? null : baseConfig.reportPath,
		};
	});
}

function buildReport({status, config, checks = [], error = null, skippedReason = null, context = null}) {
	const now = Date.now();
	const strictStartedAtMs = context?.strictStartedAtMs ?? context?.startedAtMs ?? now;
	const strictEndedAtMs = context?.strictEndedAtMs ?? now;
	const subscriberState = context?.subscriberState;
	const subscriber = context?.subscriber;
	const publisherState = context?.publisherState;
	const secondaryPublisherState = context?.secondaryPublisherState;
	const capabilities = typeof getHardwareEncoderCapabilities === 'function' ? getHardwareEncoderCapabilities() : null;
	const strictVideoSelector = primaryScreenVideoSelector(config);
	const inboundAudioFrames = subscriber ? subscriber.inboundAudioFrames() : 0;
	const inboundVideoFrames = subscriber ? subscriber.inboundVideoFrames() : 0;
	const strictVideoFrameTimes = subscriberState
		? selectedVideoFrameTimes(subscriberState, strictStartedAtMs, strictEndedAtMs, strictVideoSelector)
		: [];
	const strictVideoFrames = strictVideoFrameTimes.length;
	const strictAudioFrames = subscriberState
		? countWindowSamples(subscriberState.audioFrameTimes, strictStartedAtMs, strictEndedAtMs)
		: 0;
	const measuredFps = receivedFpsFromTimes(strictVideoFrameTimes);
	const avDriftMs = subscriberState
		? currentAvDriftMs(subscriberState, strictStartedAtMs, strictEndedAtMs, strictVideoSelector)
		: null;
	const observedPacketLoss = maxObservedPacketsLost(publisherState, secondaryPublisherState, subscriberState);
	const observedPacketLossDelta = maxObservedPacketsLostDelta(
		strictStartedAtMs,
		strictEndedAtMs,
		publisherState,
		secondaryPublisherState,
		subscriberState,
	);
	return {
		status,
		startedAt: context?.startedAtIso ?? new Date(now).toISOString(),
		endedAt: new Date(now).toISOString(),
		platform: process.platform,
		arch: process.arch,
		config: sanitizeConfig(config),
		checks,
		skippedReason,
		error: error ? String(error.stack ?? error.message ?? error) : null,
		hardwareEncoder: capabilities,
		metrics: {
			inboundAudioFrames,
			inboundVideoFrames,
			strictAudioFrames,
			strictVideoFrames,
			videoCallbacks: subscriberState?.videoCallbacks ?? 0,
			videoBytes: subscriberState?.videoBytes ?? 0,
			receivedFps: Math.round(measuredFps * 100) / 100,
			requiredFps: config.minFpsRatio > 0 ? Math.round(config.fps * config.minFpsRatio * 100) / 100 : null,
			maxVideoFrameGapMs: maxGapMs(strictVideoFrameTimes, strictStartedAtMs, strictEndedAtMs),
			maxAudioFrameGapMs: subscriberState
				? maxGapMs(subscriberState.audioFrameTimes, strictStartedAtMs, strictEndedAtMs)
				: 0,
			maxVideoFrameGapOverallMs: subscriberState?.maxVideoFrameGapMs ?? 0,
			maxAudioFrameGapOverallMs: subscriberState?.maxAudioFrameGapMs ?? 0,
			avDriftMs,
			maxObservedPacketLoss: observedPacketLoss,
			maxObservedPacketLossDelta: observedPacketLossDelta,
			videoResolutionCounts: videoResolutionCounts(
				subscriberState,
				strictStartedAtMs,
				strictEndedAtMs,
				strictVideoSelector,
			),
			videoResolutionMismatchCount: videoResolutionMismatchCount(
				subscriberState,
				config.expectedWidth,
				config.expectedHeight,
				strictStartedAtMs,
				strictEndedAtMs,
				strictVideoSelector,
			),
			droppedVideoFrameCallbacks: subscriber ? subscriber.droppedVideoFrameCallbacks() : 0,
			publisherVideoPump: context?.videoPump?.stats?.() ?? null,
			secondaryPublisherVideoPump: context?.secondaryVideoPump?.stats?.() ?? null,
			serverPublishing: summarizeServerPublishing(context?.serverState),
			lastVideoMeta: subscriberState?.lastVideoMeta ?? null,
			lastStrictVideoMeta: lastVideoFrameRecord(
				subscriberState,
				strictStartedAtMs,
				strictEndedAtMs,
				strictVideoSelector,
			),
			publisherStats: publisherState?.stats ?? null,
			secondaryPublisherStats: secondaryPublisherState?.stats ?? null,
			subscriberStats: subscriberState?.stats ?? null,
		},
	};
}

function strictFailures(report, config) {
	const failures = [];
	if (config.minFpsRatio > 0 && report.metrics.receivedFps < config.fps * config.minFpsRatio) {
		failures.push(`received FPS ${report.metrics.receivedFps} below ${config.fps * config.minFpsRatio}`);
	}
	if (config.maxFrameGapMs > 0) {
		if (report.metrics.strictVideoFrames < 2) {
			failures.push(
				'video frame gap unavailable because fewer than two video frames were observed during strict window',
			);
		} else if (report.metrics.maxVideoFrameGapMs > config.maxFrameGapMs) {
			failures.push(`max video frame gap ${report.metrics.maxVideoFrameGapMs}ms exceeds ${config.maxFrameGapMs}ms`);
		}
	}
	if (config.maxAudioFrameGapMs > 0 && expectedInboundAudioTrackCount(config) > 0) {
		if (report.metrics.strictAudioFrames < 2) {
			failures.push(
				'audio frame gap unavailable because fewer than two audio frames were observed during strict window',
			);
		} else if (report.metrics.maxAudioFrameGapMs > config.maxAudioFrameGapMs) {
			failures.push(
				`max audio frame gap ${report.metrics.maxAudioFrameGapMs}ms exceeds ${config.maxAudioFrameGapMs}ms`,
			);
		}
	}
	if (config.maxAvDriftMs > 0) {
		if (report.metrics.avDriftMs === null) {
			failures.push('A/V drift unavailable because video or audio timing was not observed');
		} else if (report.metrics.avDriftMs > config.maxAvDriftMs) {
			failures.push(`A/V drift ${report.metrics.avDriftMs}ms exceeds ${config.maxAvDriftMs}ms`);
		}
	}
	if (config.maxPacketLoss !== null && report.metrics.maxObservedPacketLossDelta > config.maxPacketLoss) {
		failures.push(
			`observed packet loss delta ${report.metrics.maxObservedPacketLossDelta} exceeds ${config.maxPacketLoss}`,
		);
	}
	if (config.requireStableResolution && report.metrics.videoResolutionMismatchCount > 0) {
		failures.push(
			`observed ${report.metrics.videoResolutionMismatchCount} video frames outside ${config.expectedWidth}x${config.expectedHeight}`,
		);
	}
	if (report.metrics.droppedVideoFrameCallbacks > 0) {
		failures.push(`dropped video frame callbacks ${report.metrics.droppedVideoFrameCallbacks}`);
	}
	return failures;
}

async function writeReportIfRequested(report, reportPath) {
	if (!reportPath) return;
	await writeFile(reportPath, `${JSON.stringify(report, null, 2)}\n`, 'utf8');
}

async function runHarness(config) {
	if (!isSupported()) {
		throw new Error(`native binding unavailable: ${loadError?.message ?? 'unknown error'}`);
	}

	const target = liveKitTcpTarget(config.url);
	const probe = await probeTcp(target, config.probeTimeoutMs);
	if (!probe.ok) {
		const message = `LiveKit is unreachable at ${config.url} (${probe.reason})`;
		if (!config.required) {
			console.log(`[livekit-harness] SKIP: ${message}. Set FLUXER_WEBRTC_SENDER_LIVEKIT_REQUIRED=1 to fail instead.`);
			return {skipped: true, skippedReason: message};
		}
		throw new Error(message);
	}

	const publisherIdentity = config.publisherIdentity;
	const secondaryPublisherIdentity = config.secondaryPublisherIdentity;
	const subscriberIdentity = config.subscriberIdentity;
	const publisherToken =
		config.publisherToken ??
		createLiveKitToken({
			apiKey: config.apiKey,
			apiSecret: config.apiSecret,
			room: config.room,
			identity: publisherIdentity,
			name: 'Fluxer Native Publisher',
		});
	const secondaryPublisherToken = config.secondaryPublisher
		? (config.secondaryPublisherToken ??
			createLiveKitToken({
				apiKey: config.apiKey,
				apiSecret: config.apiSecret,
				room: config.room,
				identity: secondaryPublisherIdentity,
				name: 'Fluxer Native Publisher 2',
			}))
		: null;
	const subscriberToken =
		config.subscriberToken ??
		createLiveKitToken({
			apiKey: config.apiKey,
			apiSecret: config.apiSecret,
			room: config.room,
			identity: subscriberIdentity,
			name: 'Fluxer Native Subscriber',
		});

	const publisher = new VoiceEngine();
	const secondaryPublisher = config.secondaryPublisher ? new VoiceEngine() : null;
	const subscriber = new VoiceEngine();
	const publisherState = createEngineState('publisher', config.verbose);
	const secondaryPublisherState = config.secondaryPublisher ? createEngineState('publisher-2', config.verbose) : null;
	const subscriberState = createEngineState('subscriber', config.verbose);
	const pumpErrors = [];
	const context = {
		publisher,
		secondaryPublisher,
		subscriber,
		publisherState,
		secondaryPublisherState,
		subscriberState,
		serverState: {
			participants: null,
			error: null,
			fetchedAtMs: null,
			lastAttemptAtMs: 0,
			inFlight: false,
		},
		config,
		publisherIdentity,
		secondaryPublisherIdentity,
		subscriberIdentity,
		startedAtMs: Date.now(),
		startedAtIso: new Date().toISOString(),
		strictStartedAtMs: null,
		strictEndedAtMs: null,
	};
	let videoPump = null;
	let secondaryVideoPump = null;
	let micPump = null;
	let secondaryMicPump = null;
	let screenAudioPump = null;
	let secondaryScreenAudioPump = null;
	let audioSampler = null;

	publisher.setEventCallback((...args) => {
		const [eventType, payload] = unpackNapiPair(args);
		publisherState.recordEvent(eventType, payload);
	});
	secondaryPublisher?.setEventCallback((...args) => {
		const [eventType, payload] = unpackNapiPair(args);
		secondaryPublisherState.recordEvent(eventType, payload);
	});
	subscriber.setEventCallback((...args) => {
		const [eventType, payload] = unpackNapiPair(args);
		subscriberState.recordEvent(eventType, payload);
	});
	subscriber.setVideoFrameCallback((...args) => {
		const [metaJson, data] = unpackNapiPair(args);
		subscriberState.recordVideoFrame(metaJson, data);
	});
	subscriber.setCountInboundAudio(true);
	audioSampler = startAudioSampler(subscriber, subscriberState);

	try {
		const e2eeKey = config.e2eeKey ? Buffer.from(config.e2eeKey, 'utf8') : undefined;
		console.log(`[livekit-harness] connecting to ${config.url} room=${config.room}`);
		await withTimeout(
			Promise.all(
				[
					publisher.connect(config.url, publisherToken, e2eeKey),
					secondaryPublisher && secondaryPublisherToken
						? secondaryPublisher.connect(config.url, secondaryPublisherToken, e2eeKey)
						: null,
					subscriber.connect(config.url, subscriberToken, e2eeKey),
				].filter(Boolean),
			),
			config.connectTimeoutMs,
			'connect',
		);

		await withTimeout(
			publisher.publishScreenShare(
				config.width,
				config.height,
				config.codec,
				config.maxBitrateBps,
				config.fps,
				config.screenSimulcast,
				{
					adaptiveSend: config.adaptiveSend,
					minVideoFps: config.minVideoFps,
					minResolutionScale: config.minResolutionScale,
					maxAudioBufferMs: config.maxAudioBufferMs,
					captureId: 'livekit-harness-primary-screen',
					trackName: 'livekit-harness-primary-screen',
				},
			),
			10_000,
			'publish screenshare',
		);
		if (secondaryPublisher) {
			await withTimeout(
				secondaryPublisher.publishScreenShare(
					config.width,
					config.height,
					config.secondaryPublisherCodec,
					config.maxBitrateBps,
					config.fps,
					config.secondaryPublisherScreenSimulcast,
					{
						adaptiveSend: config.adaptiveSend,
						minVideoFps: config.minVideoFps,
						minResolutionScale: config.minResolutionScale,
						maxAudioBufferMs: config.maxAudioBufferMs,
						captureId: 'livekit-harness-secondary-screen',
						trackName: 'livekit-harness-secondary-screen',
					},
				),
				10_000,
				'publish secondary screenshare',
			);
		}
		if (config.microphone) {
			await withTimeout(publisher.publishMicrophone(config.sampleRate, config.channels), 10_000, 'publish microphone');
		}
		if (secondaryPublisher && config.secondaryPublisherMicrophone) {
			await withTimeout(
				secondaryPublisher.publishMicrophone(config.sampleRate, config.channels),
				10_000,
				'publish secondary microphone',
			);
		}
		if (config.screenAudio) {
			await withTimeout(
				publisher.publishScreenShareAudio(config.sampleRate, config.channels),
				10_000,
				'publish screen-share audio',
			);
		}
		if (secondaryPublisher && config.secondaryPublisherScreenAudio) {
			await withTimeout(
				secondaryPublisher.publishScreenShareAudio(config.sampleRate, config.channels),
				10_000,
				'publish secondary screen-share audio',
			);
		}
		if (config.camera) {
			await withTimeout(
				publisher.publishCamera({
					deviceId: config.cameraDeviceId ?? undefined,
					width: config.width,
					height: config.height,
					frameRate: config.fps,
				}),
				10_000,
				'publish camera',
			);
		}

		videoPump = startVideoPump(publisher, config, pumpErrors);
		context.videoPump = videoPump;
		if (secondaryPublisher) {
			secondaryVideoPump = startVideoPump(secondaryPublisher, config, pumpErrors);
			context.secondaryVideoPump = secondaryVideoPump;
		}
		if (config.microphone) {
			micPump = startAudioPump(
				'pushPcm',
				(buffer, sampleRate, channels) => publisher.pushPcm(buffer, sampleRate, channels),
				config,
				pumpErrors,
			);
		}
		if (secondaryPublisher && config.secondaryPublisherMicrophone) {
			secondaryMicPump = startAudioPump(
				'secondary pushPcm',
				(buffer, sampleRate, channels) => secondaryPublisher.pushPcm(buffer, sampleRate, channels),
				config,
				pumpErrors,
			);
		}
		if (config.screenAudio) {
			screenAudioPump = startAudioPump(
				'pushScreenSharePcm',
				(buffer, sampleRate, channels) => publisher.pushScreenSharePcm(buffer, sampleRate, channels),
				config,
				pumpErrors,
			);
		}
		if (secondaryPublisher && config.secondaryPublisherScreenAudio) {
			secondaryScreenAudioPump = startAudioPump(
				'secondary pushScreenSharePcm',
				(buffer, sampleRate, channels) => secondaryPublisher.pushScreenSharePcm(buffer, sampleRate, channels),
				config,
				pumpErrors,
			);
		}
		if (config.dataPacket) {
			await publisher.publishData(Buffer.from('fluxer-native-media-harness'), {
				reliable: true,
				topic: 'native-media-harness',
				destinationIdentities: [subscriberIdentity],
			});
			if (secondaryPublisher) {
				await secondaryPublisher.publishData(Buffer.from('fluxer-native-media-harness-secondary'), {
					reliable: true,
					topic: 'native-media-harness-secondary',
					destinationIdentities: [subscriberIdentity],
				});
			}
		}
		await setSubscriberVideoQuality(subscriber, publisherIdentity, config.subscriberVideoQuality);
		if (secondaryPublisher) {
			await setSubscriberVideoQuality(subscriber, secondaryPublisherIdentity, config.subscriberVideoQuality);
		}
		if (config.subscriptionCycle) {
			await runSubscriptionCycle(subscriber, publisherIdentity);
			if (secondaryPublisher) {
				await runSubscriptionCycle(subscriber, secondaryPublisherIdentity);
			}
		}

		const checks = await waitForHarnessSuccess(context, pumpErrors, config.timeoutMs);
		await holdStrictDuration(context, pumpErrors, config.durationMs);
		const report = buildReport({status: 'pass', config, checks, context});
		const failures = strictFailures(report, config);
		if (failures.length > 0) {
			const failedReport = {...report, status: 'fail', error: failures.join('; ')};
			await writeReportIfRequested(failedReport, config.reportPath);
			const error = new Error(failures.join('; '));
			error.report = failedReport;
			throw error;
		}
		await writeReportIfRequested(report, config.reportPath);
		const producedFps = report.metrics.publisherVideoPump?.producedFps ?? null;
		console.log(
			`[livekit-harness] PASS: ${checks.length} checks, inboundAudioFrames=${report.metrics.inboundAudioFrames}, inboundVideoFrames=${report.metrics.inboundVideoFrames}, videoCallbacks=${report.metrics.videoCallbacks}, receivedFps=${report.metrics.receivedFps}, producedFps=${producedFps}, maxVideoFrameGapMs=${report.metrics.maxVideoFrameGapMs}, maxAudioFrameGapMs=${report.metrics.maxAudioFrameGapMs}, maxObservedPacketLoss=${report.metrics.maxObservedPacketLoss}, maxObservedPacketLossDelta=${report.metrics.maxObservedPacketLossDelta}, resolutionMismatches=${report.metrics.videoResolutionMismatchCount}, droppedVideoFrameCallbacks=${report.metrics.droppedVideoFrameCallbacks}`,
		);
		return {skipped: false, report};
	} catch (error) {
		if (!error.report) {
			let checks = [];
			try {
				checks = currentChecks(context);
			} catch {}
			error.report = buildReport({status: 'fail', config, checks, error, context});
		}
		throw error;
	} finally {
		videoPump?.stop();
		secondaryVideoPump?.stop();
		micPump?.stop();
		secondaryMicPump?.stop();
		screenAudioPump?.stop();
		secondaryScreenAudioPump?.stop();
		audioSampler?.stop();
		await micPump?.done();
		await secondaryMicPump?.done();
		await screenAudioPump?.done();
		await secondaryScreenAudioPump?.done();
		await disconnectEngine(subscriber, 'subscriber', config.disconnectTimeoutMs);
		await disconnectEngine(secondaryPublisher, 'publisher-2', config.disconnectTimeoutMs);
		await disconnectEngine(publisher, 'publisher', config.disconnectTimeoutMs);
	}
}

function buildAggregateReport({status, config, scenarioReports, startedAtMs}) {
	return {
		status,
		startedAt: new Date(startedAtMs).toISOString(),
		endedAt: new Date().toISOString(),
		platform: process.platform,
		arch: process.arch,
		config: sanitizeConfig(config),
		scenarioCount: scenarioReports.length,
		scenarios: scenarioReports,
	};
}

async function runScenarioSuite(baseConfig, scenarios) {
	const startedAtMs = Date.now();
	const scenarioReports = [];
	for (let index = 0; index < scenarios.length; index += 1) {
		const scenario = scenarios[index];
		console.log(
			`[livekit-harness] scenario ${index + 1}/${scenarios.length}: ${scenario.scenarioName} room=${scenario.room}`,
		);
		try {
			const result = await runHarness(scenario);
			if (result.skipped) {
				scenarioReports.push(buildReport({status: 'skip', config: scenario, skippedReason: result.skippedReason}));
			} else {
				scenarioReports.push(result.report);
			}
		} catch (error) {
			scenarioReports.push(error.report ?? buildReport({status: 'fail', config: scenario, error}));
		}
	}
	const failed = scenarioReports.filter((report) => report.status === 'fail');
	const skipped = scenarioReports.filter((report) => report.status === 'skip');
	const status = failed.length > 0 ? 'fail' : skipped.length === scenarioReports.length ? 'skip' : 'pass';
	const aggregate = buildAggregateReport({status, config: baseConfig, scenarioReports, startedAtMs});
	await writeReportIfRequested(aggregate, baseConfig.reportPath);
	if (status === 'fail') {
		const failedNames = failed.map((report) => report.config.scenarioName ?? report.config.codec).join(', ');
		const error = new Error(`LiveKit scenario suite failed: ${failedNames}`);
		error.report = aggregate;
		throw error;
	}
	return aggregate;
}

async function main() {
	const baseConfig = parseConfig();
	const scenarios = buildScenarioConfigs(baseConfig);
	try {
		if (scenarios.length > 1) {
			const aggregate = await runScenarioSuite(baseConfig, scenarios);
			console.log(`[livekit-harness] ${aggregate.status.toUpperCase()}: ${aggregate.scenarioCount} codec scenarios`);
			return 0;
		}
		const config = scenarios[0];
		const result = await runHarness(config);
		if (result.skipped) {
			const report = buildReport({status: 'skip', config, skippedReason: result.skippedReason});
			await writeReportIfRequested(report, baseConfig.reportPath);
		}
		return 0;
	} catch (error) {
		const report = error.report ?? buildReport({status: 'fail', config: baseConfig, error});
		await writeReportIfRequested(report, baseConfig.reportPath);
		console.error(`[livekit-harness] FAIL: ${error.stack ?? error.message}`);
		return 1;
	}
}

export {
	buildAggregateReport,
	buildReport,
	buildScenarioConfigs,
	createLiveKitAdminToken,
	createLiveKitToken,
	expectedCodecMime,
	jwtSubject,
	liveKitApiUrl,
	liveKitTcpTarget,
	parseCodecList,
	parseConfig,
	sanitizeConfig,
	scenarioRoomName,
	serverPublishedTrackChecks,
	statsHasOutbound,
	statsHasOutboundCodec,
	strictFailures,
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	const exitCode = await main();
	setImmediate(() => process.exit(exitCode));
}
