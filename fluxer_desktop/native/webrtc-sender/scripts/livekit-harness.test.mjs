// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {describe, test} from 'node:test';
import {
	buildReport,
	buildScenarioConfigs,
	createLiveKitAdminToken,
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
} from './livekit-harness.mjs';

function tokenForSubject(subject) {
	const payload = Buffer.from(JSON.stringify({sub: subject}), 'utf8').toString('base64url');
	return `header.${payload}.signature`;
}

function jwtPayload(token) {
	const parts = token.split('.');
	return JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
}

async function withHarnessEnv(env, callback) {
	const previous = {...process.env};
	for (const key of Object.keys(process.env)) {
		if (
			key.startsWith('LIVEKIT_') ||
			key.startsWith('FLUXER_WEBRTC_SENDER_LIVEKIT') ||
			key.startsWith('FLUXER_NATIVE_MEDIA')
		) {
			delete process.env[key];
		}
	}
	Object.assign(process.env, env);
	try {
		return await callback();
	} finally {
		for (const key of Object.keys(process.env)) {
			delete process.env[key];
		}
		Object.assign(process.env, previous);
	}
}

function strictReportFor(
	config,
	{
		videoFrameTimes = null,
		videoFrameRecords = null,
		audioFrameTimes = [],
		droppedVideoFrameCallbacks = 0,
		subscriberStatsSamples = [],
	} = {},
) {
	const strictStartedAtMs = 1_000;
	const strictEndedAtMs = 2_000;
	const effectiveVideoFrameRecords =
		videoFrameRecords ??
		(videoFrameTimes ?? []).map((atMs) => ({
			atMs,
			identity: config.publisherIdentity,
			kind: 'video',
			source: 'screen_share',
			trackName: 'screen',
			trackSid: 'TR_screen',
			width: config.expectedWidth,
			height: config.expectedHeight,
		}));
	const effectiveVideoFrameTimes = videoFrameTimes ?? effectiveVideoFrameRecords.map((record) => record.atMs);
	return buildReport({
		status: 'pass',
		config,
		context: {
			startedAtMs: strictStartedAtMs - 500,
			startedAtIso: new Date(strictStartedAtMs - 500).toISOString(),
			strictStartedAtMs,
			strictEndedAtMs,
			publisherState: {statsSamples: [], stats: null},
			subscriberState: {
				statsSamples: subscriberStatsSamples,
				stats: subscriberStatsSamples.at(-1)?.payload ?? null,
				videoCallbacks: effectiveVideoFrameTimes.length,
				videoBytes: effectiveVideoFrameTimes.length * 4,
				videoFrameTimes: effectiveVideoFrameTimes,
				videoFrameRecords: effectiveVideoFrameRecords,
				audioFrameTimes,
				lastVideoMeta: effectiveVideoFrameRecords.at(-1) ?? null,
				lastVideoFrameAtMs: effectiveVideoFrameTimes.at(-1) ?? null,
				lastAudioFrameAtMs: audioFrameTimes.at(-1) ?? null,
			},
			subscriber: {
				inboundAudioFrames: () => audioFrameTimes.length,
				inboundVideoFrames: () => effectiveVideoFrameTimes.length,
				droppedVideoFrameCallbacks: () => droppedVideoFrameCallbacks,
			},
		},
	});
}

function findCheck(checks, name) {
	const check = checks.find((entry) => entry.name === name);
	assert.ok(check, `missing check: ${name}`);
	return check;
}

function serverChecksFor(config, serverState) {
	return serverPublishedTrackChecks({
		config,
		serverState,
		publisherIdentity: config.publisherIdentity,
		secondaryPublisherIdentity: config.secondaryPublisherIdentity,
	});
}

describe('livekit harness setup matrix', () => {
	test('normalizes LiveKit screenshare source spelling in outbound stats', () => {
		const stats = {
			outbound: [
				{
					trackSid: 'TR_screen',
					source: 'screenshare',
					kind: 'video',
					codec: 'video/VP8',
					bitrateKbps: 400,
					packetsLost: 0,
				},
			],
		};

		assert.equal(statsHasOutbound(stats, 'video', 'screen_share'), true);
		assert.equal(statsHasOutboundCodec(stats, 'video', 'screen_share', 'video/VP8'), true);
		assert.equal(statsHasOutboundCodec(stats, 'video', 'screen_share', 'video/H264'), false);
	});

	test('maps all supported codec spellings to expected mime types', () => {
		assert.equal(expectedCodecMime('vp8'), 'video/VP8');
		assert.equal(expectedCodecMime('VP9'), 'video/VP9');
		assert.equal(expectedCodecMime('h264'), 'video/H264');
		assert.equal(expectedCodecMime('av1'), 'video/AV1');
		assert.equal(expectedCodecMime('h265'), 'video/H265');
		assert.equal(expectedCodecMime('hevc'), 'video/H265');
		assert.equal(expectedCodecMime(''), null);
		assert.throws(() => expectedCodecMime('h266'), /unsupported/);
	});

	test('parses codec lists with fallback and rejects empty effective lists', () => {
		assert.deepEqual(parseCodecList(' vp8, h264,, hevc ', 'av1'), ['vp8', 'h264', 'hevc']);
		assert.deepEqual(parseCodecList('', 'vp9'), ['vp9']);
		assert.throws(() => parseCodecList('', ''), /at least one/);
		assert.throws(() => parseCodecList('vp8,h266', 'vp8'), /unsupported/);
	});

	test('default non-strict config keeps the fast local live setup small', async () => {
		await withHarnessEnv({}, () => {
			const config = parseConfig();
			assert.equal(config.url, 'ws://localhost:7880');
			assert.equal(config.serverApiUrl, 'http://localhost:7880');
			assert.deepEqual(config.screenCodecs, ['vp8']);
			assert.equal(config.expectedScreenCodec, 'video/VP8');
			assert.equal(config.secondaryPublisher, false);
			assert.equal(config.microphone, true);
			assert.equal(config.screenAudio, false);
			assert.equal(config.dataPacket, false);
			assert.equal(config.subscriptionCycle, false);
			assert.equal(config.validateServerPublishing, false);
			assert.equal(config.durationMs, 0);
			assert.equal(config.externalTokens, false);
			assert.equal(config.screenSimulcast, true);
			assert.equal(config.secondaryPublisherScreenSimulcast, true);
			assert.equal(config.adaptiveSend, true);
			assert.equal(config.minVideoFps, 15);
			assert.equal(config.minResolutionScale, 0.5);
			assert.equal(config.maxAudioBufferMs, 750);
		});
	});

	test('strict config expands to multi-codec secondary-publisher scenarios', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_HARNESS_STRICT: '1',
				LIVEKIT_ROOM: 'matrix-room',
				LIVEKIT_SCREEN_CODECS: 'vp8,h264,hevc',
				LIVEKIT_EXPECT_SCREEN_CODECS: 'vp8,h264,h265',
			},
			() => {
				const config = parseConfig();
				assert.equal(config.secondaryPublisher, true);
				assert.equal(config.screenAudio, true);
				assert.equal(config.dataPacket, true);
				assert.equal(config.subscriptionCycle, true);
				assert.equal(config.validateServerPublishing, true);
				assert.equal(config.durationMs, 10 * 60 * 1000);
				assert.equal(config.maxAudioFrameGapMs, 250);
				assert.equal(config.maxPacketLoss, null);
				assert.equal(config.requireStableResolution, true);
				assert.equal(config.videoPattern, 'gradient');

				const scenarios = buildScenarioConfigs(config);
				assert.deepEqual(
					scenarios.map((scenario) => scenario.scenarioName),
					['codec-vp8-with-h264', 'codec-h264-with-hevc', 'codec-hevc-with-vp8'],
				);
				assert.deepEqual(
					scenarios.map((scenario) => scenario.expectedScreenCodec),
					['video/VP8', 'video/H264', 'video/H265'],
				);
				assert.deepEqual(
					scenarios.map((scenario) => scenario.expectedSecondaryPublisherCodec),
					['video/H264', 'video/H265', 'video/VP8'],
				);
				assert.equal(scenarios[0].room, 'matrix-room-codec-vp8-with-h264');
				assert.equal(scenarios[1].reportPath, null);
			},
		);
	});

	test('stress knobs parse packet loss, resolution stability, audio gap, and video pattern', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_MAX_PACKET_LOSS: '0',
				LIVEKIT_REQUIRE_STABLE_RESOLUTION: '0',
				LIVEKIT_MAX_AUDIO_FRAME_GAP_MS: '180',
				LIVEKIT_VIDEO_PATTERN: 'fast',
				LIVEKIT_VIDEO_INPUT: 'nv12',
				LIVEKIT_SUBSCRIBER_VIDEO_QUALITY: 'high',
				LIVEKIT_SCREEN_SIMULCAST: '0',
				LIVEKIT_SECOND_PUBLISHER_SCREEN_SIMULCAST: '1',
				LIVEKIT_SCREEN_FPS: '60',
				LIVEKIT_ADAPTIVE_SEND: '0',
				LIVEKIT_MIN_VIDEO_FPS: '24',
				LIVEKIT_MIN_RESOLUTION_SCALE: '0.75',
				LIVEKIT_MAX_AUDIO_BUFFER_MS: '640',
			},
			() => {
				const config = parseConfig();
				assert.equal(config.maxPacketLoss, 0);
				assert.equal(config.requireStableResolution, false);
				assert.equal(config.maxAudioFrameGapMs, 180);
				assert.equal(config.videoPattern, 'fast');
				assert.equal(config.videoInput, 'nv12');
				assert.equal(config.subscriberVideoQuality, 'high');
				assert.equal(config.screenSimulcast, false);
				assert.equal(config.secondaryPublisherScreenSimulcast, true);
				assert.equal(config.adaptiveSend, false);
				assert.equal(config.minVideoFps, 24);
				assert.equal(config.minResolutionScale, 0.75);
				assert.equal(config.maxAudioBufferMs, 640);
			},
		);
	});

	test('stress knobs reject impossible send pacing configurations', async () => {
		await withHarnessEnv({LIVEKIT_SCREEN_FPS: '0'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_SCREEN_FPS must be a positive number/),
		);
		await withHarnessEnv({LIVEKIT_MIN_VIDEO_FPS: '0'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_MIN_VIDEO_FPS must be a positive number/),
		);
		await withHarnessEnv({LIVEKIT_SCREEN_FPS: '30', LIVEKIT_MIN_VIDEO_FPS: '60'}, () =>
			assert.throws(() => parseConfig(), /MIN_VIDEO_FPS must be less than or equal/),
		);
		await withHarnessEnv({LIVEKIT_MAX_AUDIO_BUFFER_MS: '0'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_MAX_AUDIO_BUFFER_MS must be a positive integer/),
		);
		await withHarnessEnv({LIVEKIT_MIN_RESOLUTION_SCALE: '0'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_MIN_RESOLUTION_SCALE must be a positive number/),
		);
		await withHarnessEnv({LIVEKIT_MIN_RESOLUTION_SCALE: '1.5'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_MIN_RESOLUTION_SCALE must be greater than 0/),
		);
		await withHarnessEnv({LIVEKIT_ENABLE_SCREEN_AUDIO: 'maybe'}, () =>
			assert.throws(() => parseConfig(), /LIVEKIT_ENABLE_SCREEN_AUDIO must be a boolean flag/),
		);
		await withHarnessEnv({LIVEKIT_SCREEN_CODECS: 'vp8,h264', LIVEKIT_EXPECT_SCREEN_CODECS: 'vp8'}, () =>
			assert.throws(() => parseConfig(), /EXPECT_SCREEN_CODECS length must match/),
		);
	});

	test('explicit secondary codec expectations override rotating defaults', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_SCREEN_CODECS: 'vp8,h264',
				LIVEKIT_SECOND_PUBLISHER_CODEC: 'av1',
				LIVEKIT_EXPECT_SECOND_PUBLISHER_SCREEN_CODEC: 'hevc',
			},
			() => {
				const scenarios = buildScenarioConfigs(parseConfig());
				assert.deepEqual(
					scenarios.map((scenario) => scenario.secondaryPublisherCodec),
					['av1', 'av1'],
				);
				assert.deepEqual(
					scenarios.map((scenario) => scenario.expectedSecondaryPublisherCodec),
					['video/H265', 'video/H265'],
				);
			},
		);
	});

	test('secondary screen codec aliases are accepted and conflicting aliases are rejected', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_SCREEN_CODECS: 'h264',
				LIVEKIT_SECOND_PUBLISHER_SCREEN_CODECS: 'vp8',
				LIVEKIT_EXPECT_SECONDARY_PUBLISHER_SCREEN_CODEC: 'vp8',
			},
			() => {
				const config = parseConfig();
				assert.equal(config.secondaryPublisherCodec, 'vp8');
				assert.equal(config.expectedSecondaryPublisherCodec, 'video/VP8');
			},
		);

		await withHarnessEnv(
			{
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_SECOND_PUBLISHER_CODEC: 'h264',
				LIVEKIT_SECOND_PUBLISHER_SCREEN_CODEC: 'vp8',
			},
			() => assert.throws(() => parseConfig(), /conflicts with LIVEKIT_SECOND_PUBLISHER_CODEC/),
		);

		await withHarnessEnv(
			{
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_EXPECT_SECOND_PUBLISHER_SCREEN_CODEC: 'h264',
				LIVEKIT_EXPECT_SECONDARY_PUBLISHER_SCREEN_CODEC: 'vp8',
			},
			() => assert.throws(() => parseConfig(), /conflicts with LIVEKIT_EXPECT_SECOND_PUBLISHER_SCREEN_CODEC/),
		);
	});

	test('external token setup derives identities, requires matching tokens, and redacts reports', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_PUBLISHER_TOKEN: tokenForSubject('publisher-subject'),
			},
			() => assert.throws(() => parseConfig(), /both required/),
		);

		await withHarnessEnv(
			{
				LIVEKIT_ROOM: 'shared-token-room',
				LIVEKIT_SCREEN_CODECS: 'vp8,h264',
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_PUBLISHER_TOKEN: tokenForSubject('publisher-subject'),
				LIVEKIT_SUBSCRIBER_TOKEN: tokenForSubject('subscriber-subject'),
			},
			() => assert.throws(() => parseConfig(), /SECONDARY_PUBLISHER_TOKEN/),
		);

		await withHarnessEnv(
			{
				LIVEKIT_ROOM: 'shared-token-room',
				LIVEKIT_SCREEN_CODECS: 'vp8,h264',
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_PUBLISHER_TOKEN: tokenForSubject('publisher-subject'),
				LIVEKIT_SUBSCRIBER_TOKEN: tokenForSubject('subscriber-subject'),
				LIVEKIT_SECONDARY_PUBLISHER_TOKEN: tokenForSubject('secondary-subject'),
				LIVEKIT_E2EE_KEY: 'secret-key',
			},
			() => {
				const config = parseConfig();
				assert.equal(config.externalTokens, true);
				assert.equal(config.publisherIdentity, 'publisher-subject');
				assert.equal(config.subscriberIdentity, 'subscriber-subject');
				assert.equal(config.secondaryPublisherIdentity, 'secondary-subject');
				assert.deepEqual(
					buildScenarioConfigs(config).map((scenario) => scenario.room),
					['shared-token-room', 'shared-token-room'],
				);

				const sanitized = sanitizeConfig(config);
				assert.equal(sanitized.url, 'ws://localhost:7880');
				assert.equal(sanitized.apiSecret, '<redacted>');
				assert.equal(sanitized.e2eeKey, '<present>');
				assert.equal(sanitized.publisherToken, '<present>');
				assert.equal(sanitized.subscriberToken, '<present>');
				assert.equal(sanitized.secondaryPublisherToken, '<present>');
			},
		);
	});

	test('report sanitization strips URL credentials, query strings, and fragments', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_URL: 'wss://user:password@example.test:443/rtc?access_token=secret#fragment',
			},
			() => {
				const sanitized = sanitizeConfig(parseConfig());
				assert.equal(sanitized.url, 'wss://example.test/rtc');
				assert.doesNotMatch(JSON.stringify(sanitized), /user|password|access_token|secret|fragment/);
			},
		);
	});

	test('jwt subject and TCP target parsing catch malformed live setups early', () => {
		assert.equal(jwtSubject(tokenForSubject('user_1_connection')), 'user_1_connection');
		assert.throws(() => jwtSubject('not-a-jwt'), /must be a JWT/);
		assert.throws(() => jwtSubject('header.bad-json.signature'), /failed to decode/);
		assert.throws(() => jwtSubject(tokenForSubject('')), /does not contain/);

		assert.deepEqual(liveKitTcpTarget('ws://localhost:7880'), {host: 'localhost', port: 7880});
		assert.deepEqual(liveKitTcpTarget('wss://[::1]/rtc'), {host: '::1', port: 443});
		assert.equal(liveKitApiUrl('ws://localhost:7880/rtc?token=secret'), 'http://localhost:7880');
		assert.equal(liveKitApiUrl('wss://livekit.example.test/rtc'), 'https://livekit.example.test');
		assert.throws(() => liveKitTcpTarget('https://localhost'), /must use ws/);
	});

	test('admin token has the room-scoped grant needed for server publishing validation', () => {
		const token = createLiveKitAdminToken({apiKey: 'devkey', apiSecret: 'secret', room: 'room-a'});
		const payload = jwtPayload(token);
		assert.equal(payload.iss, 'devkey');
		assert.equal(payload.video.room, 'room-a');
		assert.equal(payload.video.roomAdmin, true);
		assert.equal(payload.video.roomJoin, undefined);
	});

	test('scenario room names are stable and bounded for generated-token suites', () => {
		assert.equal(scenarioRoomName('room', 'codec-vp8', 1, false), 'room');
		assert.equal(scenarioRoomName('room', 'codec:vP8 with H264', 2, false), 'room-codec-vP8-with-H264');
		assert.equal(scenarioRoomName('room', 'codec-vp8', 2, true), 'room');

		const longRoom = scenarioRoomName('r'.repeat(120), 'codec-h264-with-hevc', 2, false);
		assert.equal(longRoom.length, 128);
		assert.ok(longRoom.startsWith('r'.repeat(120)));
	});

	test('strict report checks fail when the strict window has too few samples to measure gaps', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_MIN_RECEIVED_FPS_RATIO: '0',
				LIVEKIT_MAX_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AUDIO_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AV_DRIFT_MS: '0',
			},
			() => {
				const config = parseConfig();
				const report = strictReportFor(config, {
					videoFrameTimes: [1_250],
					audioFrameTimes: [1_300],
				});
				assert.equal(report.metrics.strictVideoFrames, 1);
				assert.equal(report.metrics.strictAudioFrames, 1);
				assert.match(strictFailures(report, config).join('\n'), /video frame gap unavailable/);
				assert.match(strictFailures(report, config).join('\n'), /audio frame gap unavailable/);
			},
		);
	});

	test('strict report checks use only samples inside the measured window', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_MIN_RECEIVED_FPS_RATIO: '0',
				LIVEKIT_MAX_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AUDIO_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AV_DRIFT_MS: '0',
			},
			() => {
				const config = parseConfig();
				const report = strictReportFor(config, {
					videoFrameTimes: [500, 1_050, 1_200, 2_500],
					audioFrameTimes: [600, 1_100, 1_240, 2_600],
				});
				assert.equal(report.metrics.strictVideoFrames, 2);
				assert.equal(report.metrics.strictAudioFrames, 2);
				assert.equal(report.metrics.maxVideoFrameGapMs, 150);
				assert.equal(report.metrics.maxAudioFrameGapMs, 140);
				assert.deepEqual(strictFailures(report, config), []);
			},
		);
	});

	test('strict screen report ignores camera video callbacks for resolution and drift', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_ENABLE_CAMERA: '1',
				LIVEKIT_MIN_RECEIVED_FPS_RATIO: '0',
				LIVEKIT_MAX_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AUDIO_FRAME_GAP_MS: '250',
				LIVEKIT_MAX_AV_DRIFT_MS: '80',
			},
			() => {
				const config = parseConfig();
				const report = strictReportFor(config, {
					videoFrameRecords: [
						{
							atMs: 1_010,
							identity: config.publisherIdentity,
							kind: 'video',
							source: 'screen_share',
							trackName: 'screen',
							trackSid: 'TR_screen',
							width: 320,
							height: 180,
						},
						{
							atMs: 1_025,
							identity: config.publisherIdentity,
							kind: 'video',
							source: 'camera',
							trackName: 'camera',
							trackSid: 'TR_camera',
							width: 480,
							height: 360,
						},
						{
							atMs: 1_080,
							identity: config.publisherIdentity,
							kind: 'video',
							source: 'screen_share',
							trackName: 'screen',
							trackSid: 'TR_screen',
							width: 320,
							height: 180,
						},
						{
							atMs: 1_220,
							identity: config.publisherIdentity,
							kind: 'video',
							source: 'camera',
							trackName: 'camera',
							trackSid: 'TR_camera',
							width: 480,
							height: 360,
						},
					],
					audioFrameTimes: [1_040, 1_100],
				});

				assert.equal(report.metrics.videoCallbacks, 4);
				assert.equal(report.metrics.strictVideoFrames, 2);
				assert.deepEqual(report.metrics.videoResolutionCounts, {'320x180': 2});
				assert.equal(report.metrics.videoResolutionMismatchCount, 0);
				assert.equal(report.metrics.avDriftMs, 20);
				assert.deepEqual(strictFailures(report, config), []);
			},
		);
	});

	test('strict packet loss gate uses loss deltas inside the strict window', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_MIN_RECEIVED_FPS_RATIO: '0',
				LIVEKIT_MAX_FRAME_GAP_MS: '0',
				LIVEKIT_MAX_AUDIO_FRAME_GAP_MS: '0',
				LIVEKIT_MAX_AV_DRIFT_MS: '0',
				LIVEKIT_MAX_PACKET_LOSS: '0',
			},
			() => {
				const config = parseConfig();
				const report = strictReportFor(config, {
					videoFrameTimes: [1_100, 1_200],
					subscriberStatsSamples: [
						{atMs: 900, payload: {outbound: [], inbound: [{kind: 'video', packetsLost: 4}]}},
						{atMs: 1_500, payload: {outbound: [], inbound: [{kind: 'video', packetsLost: 4}]}},
					],
				});
				assert.equal(report.metrics.maxObservedPacketLoss, 4);
				assert.equal(report.metrics.maxObservedPacketLossDelta, 0);
				assert.deepEqual(strictFailures(report, config), []);

				const failingReport = strictReportFor(config, {
					videoFrameTimes: [1_100, 1_200],
					subscriberStatsSamples: [
						{atMs: 900, payload: {outbound: [], inbound: [{kind: 'video', packetsLost: 4}]}},
						{atMs: 1_500, payload: {outbound: [], inbound: [{kind: 'video', packetsLost: 5}]}},
					],
				});
				assert.equal(failingReport.metrics.maxObservedPacketLossDelta, 1);
				assert.match(strictFailures(failingReport, config).join('\n'), /packet loss delta 1/);
			},
		);
	});

	test('server publishing checks cover primary and secondary screen, audio, and camera tracks', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1',
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_ENABLE_SCREEN_AUDIO: '1',
				LIVEKIT_ENABLE_CAMERA: '1',
			},
			() => {
				const config = parseConfig();
				const serverState = {
					participants: [
						{
							identity: config.publisherIdentity,
							tracks: [
								{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false, sid: 'TR_V1'},
								{type: 'AUDIO', source: 'MICROPHONE', mimeType: 'audio/red', muted: false, sid: 'TR_A1'},
								{type: 'AUDIO', source: 'SCREEN_SHARE_AUDIO', mimeType: 'audio/red', muted: false, sid: 'TR_A2'},
								{type: 'VIDEO', source: 'CAMERA', muted: false, sid: 'TR_V2'},
							],
						},
						{
							identity: config.secondaryPublisherIdentity,
							tracks: [
								{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false, sid: 'TR_V3'},
								{type: 'AUDIO', source: 'MICROPHONE', mimeType: 'audio/red', muted: false, sid: 'TR_A3'},
								{type: 'AUDIO', source: 'SCREEN_SHARE_AUDIO', mimeType: 'audio/red', muted: false, sid: 'TR_A4'},
							],
						},
					],
					error: null,
				};
				const checks = serverChecksFor(config, serverState);
				assert.equal(checks.length, 8);
				assert.deepEqual(
					checks.map((check) => [check.name, check.pass]),
					[
						['server publishing API participants listed', true],
						['server sees publisher screenshare publication', true],
						['server sees publisher microphone publication', true],
						['server sees publisher screen-share audio publication', true],
						['server sees publisher camera publication', true],
						['server sees secondary publisher screenshare publication', true],
						['server sees secondary publisher microphone publication', true],
						['server sees secondary publisher screen-share audio publication', true],
					],
				);
			},
		);
	});

	test('server publishing checks fail closed on missing server tracks', async () => {
		await withHarnessEnv({LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1', LIVEKIT_ENABLE_SCREEN_AUDIO: '1'}, () => {
			const config = parseConfig();
			const checks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(checks, 'server sees publisher screenshare publication').pass, true);
			assert.equal(findCheck(checks, 'server sees publisher screen-share audio publication').pass, false);
		});
	});

	test('server publishing checks reject wrong participant identity', async () => {
		await withHarnessEnv({LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1'}, () => {
			const config = parseConfig();
			const checks = serverChecksFor(config, {
				participants: [
					{
						identity: config.subscriberIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(checks, 'server publishing API participants listed').pass, true);
			assert.equal(findCheck(checks, 'server sees publisher screenshare publication').pass, false);
		});
	});

	test('server publishing checks reject wrong kind, source, and screen mime', async () => {
		await withHarnessEnv({LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1', LIVEKIT_EXPECT_SCREEN_CODEC: 'h264'}, () => {
			const config = parseConfig();
			const wrongKindChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'AUDIO', source: 'SCREEN_SHARE', mimeType: 'video/H264', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(wrongKindChecks, 'server sees publisher screenshare publication').pass, false);

			const wrongSourceChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'CAMERA', mimeType: 'video/H264', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(wrongSourceChecks, 'server sees publisher screenshare publication').pass, false);

			const wrongMimeChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(wrongMimeChecks, 'server sees publisher screenshare publication').pass, false);

			const matchingChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mime_type: 'video/h264', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(matchingChecks, 'server sees publisher screenshare publication').pass, true);
		});
	});

	test('server publishing checks reject muted or missing mute state on expected tracks', async () => {
		await withHarnessEnv({LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1'}, () => {
			const config = parseConfig();
			const mutedChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: true}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(mutedChecks, 'server sees publisher screenshare publication').pass, false);

			const missingMutedChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8'}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(missingMutedChecks, 'server sees publisher screenshare publication').pass, false);
		});
	});

	test('server publishing checks handle protobuf numeric enums without accepting data tracks as video', async () => {
		await withHarnessEnv({LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1'}, () => {
			const config = parseConfig();
			const dataTrackChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 2, source: 3, mimeType: 'video/VP8', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(dataTrackChecks, 'server sees publisher screenshare publication').pass, false);

			const videoTrackChecks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [{type: 1, source: 3, mimeType: 'video/VP8', muted: false}],
					},
				],
				error: null,
			});
			assert.equal(findCheck(videoTrackChecks, 'server sees publisher screenshare publication').pass, true);
		});
	});

	test('server publishing checks fail closed on missing secondary, audio, camera, and screen-audio tracks', async () => {
		await withHarnessEnv(
			{
				LIVEKIT_VALIDATE_SERVER_PUBLISHING: '1',
				LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
				LIVEKIT_ENABLE_SCREEN_AUDIO: '1',
				LIVEKIT_ENABLE_CAMERA: '1',
			},
			() => {
				const config = parseConfig();
				const checks = serverChecksFor(config, {
					participants: [
						{
							identity: config.publisherIdentity,
							tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false}],
						},
						{
							identity: config.secondaryPublisherIdentity,
							tracks: [{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false}],
						},
					],
					error: null,
				});
				assert.equal(findCheck(checks, 'server sees publisher screenshare publication').pass, true);
				assert.equal(findCheck(checks, 'server sees publisher microphone publication').pass, false);
				assert.equal(findCheck(checks, 'server sees publisher screen-share audio publication').pass, false);
				assert.equal(findCheck(checks, 'server sees publisher camera publication').pass, false);
				assert.equal(findCheck(checks, 'server sees secondary publisher screenshare publication').pass, true);
				assert.equal(findCheck(checks, 'server sees secondary publisher microphone publication').pass, false);
				assert.equal(findCheck(checks, 'server sees secondary publisher screen-share audio publication').pass, false);
			},
		);
	});

	test('strict server publishing validation does not pass open on Twirp errors', async () => {
		await withHarnessEnv({LIVEKIT_HARNESS_STRICT: '1'}, () => {
			const config = parseConfig();
			const checks = serverChecksFor(config, {
				participants: [
					{
						identity: config.publisherIdentity,
						tracks: [
							{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false},
							{type: 'AUDIO', source: 'MICROPHONE', mimeType: 'audio/red', muted: false},
							{type: 'AUDIO', source: 'SCREEN_SHARE_AUDIO', mimeType: 'audio/red', muted: false},
						],
					},
					{
						identity: config.secondaryPublisherIdentity,
						tracks: [
							{type: 'VIDEO', source: 'SCREEN_SHARE', mimeType: 'video/VP8', muted: false},
							{type: 'AUDIO', source: 'MICROPHONE', mimeType: 'audio/red', muted: false},
							{type: 'AUDIO', source: 'SCREEN_SHARE_AUDIO', mimeType: 'audio/red', muted: false},
						],
					},
				],
				error: 'RoomService.ListParticipants failed with HTTP 401',
			});
			assert.equal(config.validateServerPublishing, true);
			assert.equal(findCheck(checks, 'server publishing API participants listed').pass, false);
			assert.equal(
				checks.every((check) => check.pass),
				false,
			);
		});
	});
});
