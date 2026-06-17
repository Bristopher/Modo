// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';

const require = createRequire(import.meta.url);
const webrtcSender = require('./index.js');

describe('webrtc-sender loader wrapper', () => {
	test('resolves the native addon filename for every desktop OS/arch target', () => {
		const cases = [
			['win32', 'x64', 'webrtc-sender.win32-x64-msvc.node'],
			['win32', 'arm64', 'webrtc-sender.win32-arm64-msvc.node'],
			['darwin', 'x64', 'webrtc-sender.darwin-x64.node'],
			['darwin', 'arm64', 'webrtc-sender.darwin-arm64.node'],
			['linux', 'x64', 'webrtc-sender.linux-x64-gnu.node'],
			['linux', 'arm64', 'webrtc-sender.linux-arm64-gnu.node'],
		];
		for (const [platform, arch, expected] of cases) {
			assert.equal(webrtcSender.__nativeFileNameForTests(platform, arch), expected);
		}
	});

	test('rejects unsupported platform/architecture pairs explicitly', () => {
		assert.throws(() => webrtcSender.__nativeFileNameForTests('linux', 'ia32'), /not supported/);
		assert.throws(() => webrtcSender.__nativeFileNameForTests('freebsd', 'x64'), /not supported/);
	});

	test('returns a native hardware encoder capability when the binding exports one', () => {
		const expected = {
			available: true,
			backend: 'nvenc',
			compiled: true,
			runtime: true,
			codecs: ['h264', 'h265'],
			zeroCopy: true,
			nativeInputs: ['dmabuf'],
		};
		webrtcSender.__setBindingForTests({
			getHardwareEncoderCapability() {
				return expected;
			},
		});

		assert.deepEqual(webrtcSender.getHardwareEncoderCapability(), expected);
		assert.deepEqual(webrtcSender.getHardwareEncoderCapabilities(), expected);
	});

	test('delegates dropped video callback metrics through the VoiceEngine wrapper', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				inboundAudioFrames() {
					return 2;
				}

				inboundVideoFrames() {
					return 3;
				}

				droppedVideoFrameCallbacks() {
					return 5;
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.equal(engine.inboundAudioFrames(), 2);
		assert.equal(engine.inboundVideoFrames(), 3);
		assert.equal(engine.droppedVideoFrameCallbacks(), 5);
	});

	test('reports VoiceEngine feature capabilities from the wrapped native prototype', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishDeviceMicrophone() {}
				publishMicrophone() {}
				publishCamera() {}
				listCameraDevices() {}
				publishScreenShare() {}
				updateScreenShareEncoding() {}
				unpublishScreenShare() {}
				publishScreenShareAudio() {}
				pushScreenSharePcm() {}
				pushScreenShareFloat() {}
				unpublishScreenShareAudio() {}
				listAudioInputDevices() {}
				listAudioOutputDevices() {}
				setAudioOutputDevice() {}
				setParticipantVolume() {}
				setRemoteTrackSubscription() {}
				publishData() {}
				getConnectionStats() {}
				setVideoFrameCallback() {}
			},
		});

		assert.deepEqual(webrtcSender.getCapabilities(), {
			microphoneCapture: true,
			syntheticMicrophonePcm: true,
			cameraCapture: true,
			nativeCameraBackgrounds: false,
			screenShare: true,
			screenShareEncodingUpdate: true,
			screenShareAudio: true,
			deviceLists: true,
			outputDeviceSelection: true,
			participantVolume: true,
			remoteTrackSubscription: true,
			dataChannel: true,
			connectionStats: true,
			nativeVideoFrames: true,
			hardwareEncoderCapabilities: true,
		});
	});

	test('does not report screen-share audio capability without the float push method', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishScreenShareAudio() {}
				pushScreenSharePcm() {}
				unpublishScreenShareAudio() {}
			},
		});

		assert.equal(webrtcSender.getCapabilities().screenShareAudio, false);
	});

	test('reports native camera background support only from the explicit native probe', () => {
		webrtcSender.__setBindingForTests({
			hasNativeCameraBackgrounds() {
				return true;
			},
			VoiceEngine: class {
				publishCamera() {}
				listCameraDevices() {}
			},
		});

		assert.equal(webrtcSender.hasNativeCameraBackgrounds(), true);
		assert.equal(webrtcSender.getCapabilities().nativeCameraBackgrounds, true);

		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishCamera() {}
				listCameraDevices() {}
			},
		});

		assert.equal(webrtcSender.hasNativeCameraBackgrounds(), false);
		assert.equal(webrtcSender.getCapabilities().nativeCameraBackgrounds, false);
	});

	test('delegates video-frame callback clearing to the native binding when exported', () => {
		let clearCalls = 0;
		const setCalls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				setVideoFrameCallback(callback) {
					setCalls.push(callback);
				}

				clearVideoFrameCallback() {
					clearCalls += 1;
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		engine.clearVideoFrameCallback();

		assert.equal(clearCalls, 1);
		assert.deepEqual(setCalls, []);
	});

	test('delegates screen-share float audio through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				pushScreenShareFloat(buffer, sampleRate, numChannels) {
					calls.push([buffer, sampleRate, numChannels]);
					return Promise.resolve(true);
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		const buffer = Buffer.from(new Float32Array([0.25, -0.25]).buffer);
		const accepted = await engine.pushScreenShareFloat(buffer, 48000, 2);

		assert.equal(accepted, true);
		assert.deepEqual(calls, [[buffer, 48000, 2]]);
	});

	test('falls back to a no-op video-frame callback for older native bindings', () => {
		const setCalls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				setVideoFrameCallback(callback) {
					setCalls.push(callback);
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		engine.clearVideoFrameCallback();

		assert.equal(setCalls.length, 1);
		assert.equal(typeof setCalls[0], 'function');
		assert.equal(setCalls[0]('{}', Buffer.alloc(0)), undefined);
	});

	test('treats video-frame callback clearing as a no-op when the binding exports neither method', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();

		assert.equal(engine.clearVideoFrameCallback(), undefined);
	});

	test('delegates engine bridge version reads and assertions to the native binding', () => {
		const assertedVersions = [];
		webrtcSender.__setBindingForTests({
			getEngineBridgeVersion() {
				return 9;
			},
			assertEngineBridgeVersion(version) {
				assertedVersions.push(version);
				if (version !== 9) {
					throw new Error(`voice engine bridge version mismatch: host sent ${version}, native addon expects 9`);
				}
			},
		});

		assert.equal(webrtcSender.getEngineBridgeVersion(), 9);
		webrtcSender.assertEngineBridgeVersion(9);
		assert.throws(() => webrtcSender.assertEngineBridgeVersion(8), /bridge version mismatch/);
		assert.deepEqual(assertedVersions, [9, 8]);
	});

	test('reports a null engine bridge version and throws on assertion when the binding lacks the exports', () => {
		webrtcSender.__setBindingForTests({});

		assert.equal(webrtcSender.getEngineBridgeVersion(), null);
		assert.throws(() => webrtcSender.assertEngineBridgeVersion(9), /does not export assertEngineBridgeVersion/);
	});

	test('delegates native voice engine prewarm when the binding exports it', () => {
		let prewarmCalls = 0;
		webrtcSender.__setBindingForTests({
			prewarmVoiceEngine() {
				prewarmCalls += 1;
			},
		});

		webrtcSender.prewarmVoiceEngine();

		assert.equal(prewarmCalls, 1);
	});

	test('delegates device microphone publish through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishDeviceMicrophone(opts) {
					calls.push(opts);
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.publishDeviceMicrophone({
			deviceId: 'mic-guid',
			echoCancellation: false,
			noiseSuppression: true,
			autoGainControl: false,
		});
		await engine.publishDeviceMicrophone({
			deviceId: 'mic-guid',
			echoCancellation: false,
			noiseSuppression: true,
			autoGainControl: false,
			maxBitrateBps: 96_000,
		});

		assert.deepEqual(calls, [
			{
				deviceId: 'mic-guid',
				echoCancellation: false,
				noiseSuppression: true,
				autoGainControl: false,
				maxBitrateBps: undefined,
			},
			{
				deviceId: 'mic-guid',
				echoCancellation: false,
				noiseSuppression: true,
				autoGainControl: false,
				maxBitrateBps: 96_000,
			},
		]);
	});

	test('delegates native camera devices and publish options through the VoiceEngine wrapper', async () => {
		const publishCalls = [];
		const processedPublishCalls = [];
		const processedFrameCalls = [];
		const nativeSinkPublishCalls = [];
		const nativeSinkHandle = {};
		const devices = [
			{
				deviceId: 'native-camera-id',
				label: 'Studio Display Camera',
				description: 'Apple Studio Display Camera',
				index: 0,
				deviceIdAliases: ['native-camera-id', '0'],
			},
		];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				listCameraDevices() {
					return devices;
				}

				publishCamera(opts) {
					publishCalls.push(opts);
					return Promise.resolve();
				}

				publishProcessedCamera(opts) {
					processedPublishCalls.push(opts);
					return Promise.resolve({trackSid: 'TR_processed_camera'});
				}

				publishNativeCameraSink(opts) {
					nativeSinkPublishCalls.push(opts);
					return Promise.resolve({trackSid: 'TR_native_camera'});
				}

				createCameraFrameSinkHandle() {
					return nativeSinkHandle;
				}

				pushProcessedCameraFrame(frame) {
					processedFrameCalls.push(frame);
					return Promise.resolve(true);
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.equal(engine.listCameraDevices(), devices);
		await engine.publishCamera({
			deviceId: 'native-camera-id',
			width: 1280,
			height: 720,
			frameRate: 30,
			mirror: true,
			codec: 'h265',
			maxBitrateBps: 6_000_000,
			maxFramerate: 30,
		});
		const processedPublishResult = await engine.publishProcessedCamera({
			width: 1280,
			height: 720,
			frameRate: 30,
			ignored: true,
		});
		const nativeSinkPublishResult = await engine.publishNativeCameraSink({
			deviceId: 'native-camera-id',
			width: 1280,
			height: 720,
			frameRate: 30,
			backgroundMode: 'custom',
			backgroundCustomMediaPath: '/tmp/bg.webp',
			backgroundCustomMediaKind: 'animated',
			codec: 'h264',
		});
		const cameraFrameSinkHandle = engine.createCameraFrameSinkHandle();
		const frame = {
			format: 'i420',
			width: 4,
			height: 2,
			timestampUs: 12_345,
			data: Buffer.alloc(12),
			ignored: true,
		};
		const processedFrameResult = await engine.pushProcessedCameraFrame(frame);

		assert.deepEqual(publishCalls, [
			{
				deviceId: 'native-camera-id',
				width: 1280,
				height: 720,
				frameRate: 30,
				mirror: true,
				backgroundMode: undefined,
				backgroundCustomMediaPath: undefined,
				backgroundCustomMediaKind: undefined,
				backgroundBlurStrength: undefined,
				codec: 'h265',
				maxBitrateBps: 6_000_000,
				maxFramerate: 30,
			},
		]);
		assert.deepEqual(processedPublishResult, {trackSid: 'TR_processed_camera'});
		assert.deepEqual(processedPublishCalls, [
			{
				width: 1280,
				height: 720,
				frameRate: 30,
			},
		]);
		assert.deepEqual(nativeSinkPublishResult, {trackSid: 'TR_native_camera'});
		assert.deepEqual(nativeSinkPublishCalls, [
			{
				deviceId: 'native-camera-id',
				width: 1280,
				height: 720,
				frameRate: 30,
				mirror: undefined,
				backgroundMode: 'custom',
				backgroundCustomMediaPath: '/tmp/bg.webp',
				backgroundCustomMediaKind: 'animated',
				backgroundBlurStrength: undefined,
				codec: 'h264',
				maxBitrateBps: undefined,
				maxFramerate: undefined,
			},
		]);
		assert.equal(cameraFrameSinkHandle, nativeSinkHandle);
		assert.equal(processedFrameResult, true);
		assert.deepEqual(processedFrameCalls, [
			{
				format: 'i420',
				width: 4,
				height: 2,
				timestampUs: 12_345,
				data: frame.data,
			},
		]);
	});

	test('reports processed camera publishing as unavailable for older native bindings', async () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.throws(
			() => engine.publishProcessedCamera({width: 1280, height: 720, frameRate: 30}),
			/native binding does not export publishProcessedCamera/,
		);
		assert.throws(
			() => engine.publishNativeCameraSink({width: 1280, height: 720, frameRate: 30}),
			/native binding does not export publishNativeCameraSink/,
		);
		assert.equal(engine.createCameraFrameSinkHandle(), null);
		assert.throws(
			() =>
				engine.pushProcessedCameraFrame({
					format: 'i420',
					width: 4,
					height: 2,
					timestampUs: 12_345,
					data: Buffer.alloc(12),
				}),
			/native binding does not export pushProcessedCameraFrame/,
		);
	});

	test('delegates device screen-share publish through the VoiceEngine wrapper', async () => {
		const publishCalls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishDeviceScreenShare(opts) {
					publishCalls.push(opts);
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.publishDeviceScreenShare({
			deviceId: 'studio-display-camera',
			width: 1920,
			height: 1080,
			frameRate: 60,
			codec: 'h264',
			maxBitrateBps: 8_000_000,
			maxFramerate: 60,
		});

		assert.deepEqual(publishCalls, [
			{
				deviceId: 'studio-display-camera',
				width: 1920,
				height: 1080,
				frameRate: 60,
				mirror: undefined,
				backgroundMode: undefined,
				backgroundCustomMediaPath: undefined,
				backgroundCustomMediaKind: undefined,
				backgroundBlurStrength: undefined,
				codec: 'h264',
				maxBitrateBps: 8_000_000,
				maxFramerate: 60,
			},
		]);
	});

	test('delegates camera capture updates with effect strengths through the VoiceEngine wrapper', async () => {
		const updateCalls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				updateCameraCapture(opts) {
					updateCalls.push(opts);
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.updateCameraCapture({
			deviceId: 'native-camera-id',
			width: 1280,
			height: 720,
			frameRate: 30,
			mirror: false,
			backgroundMode: 'blur',
			backgroundBlurStrength: 90,
			ignored: true,
		});

		assert.deepEqual(updateCalls, [
			{
				deviceId: 'native-camera-id',
				width: 1280,
				height: 720,
				frameRate: 30,
				mirror: false,
				backgroundMode: 'blur',
				backgroundCustomMediaPath: undefined,
				backgroundCustomMediaKind: undefined,
				backgroundBlurStrength: 90,
				codec: undefined,
				maxBitrateBps: undefined,
				maxFramerate: undefined,
			},
		]);
	});

	test('reports camera capture updates as unavailable for older native bindings', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.throws(
			() => engine.updateCameraCapture({deviceId: 'native-camera-id'}),
			/native binding does not export updateCameraCapture/,
		);
	});

	test('reports device screen-share publish as unavailable for older native bindings', async () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.throws(
			() => engine.publishDeviceScreenShare({deviceId: 'studio-display-camera'}),
			/native binding does not export publishDeviceScreenShare/,
		);
	});

	test('delegates screen-share simulcast selection through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishScreenShare(width, height, codec, maxBitrateBps, maxFramerate, simulcast, options) {
					calls.push({width, height, codec, maxBitrateBps, maxFramerate, simulcast, options});
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.publishScreenShare(3840, 2160, 'h265', 50_000_000, 60, false, {
			adaptiveSend: true,
			minVideoFps: 15,
			minResolutionScale: 0.5,
			maxAudioBufferMs: 750,
			captureId: 'screen-harness-primary',
		});
		assert.deepEqual(calls, [
			{
				width: 3840,
				height: 2160,
				codec: 'h265',
				maxBitrateBps: 50_000_000,
				maxFramerate: 60,
				simulcast: false,
				options: {
					adaptiveSend: true,
					minVideoFps: 15,
					minResolutionScale: 0.5,
					maxAudioBufferMs: 750,
					captureId: 'screen-harness-primary',
				},
			},
		]);
	});

	test('rejects screen-share publish and encoding update without capture IDs', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishScreenShare(...args) {
					calls.push(['publish', args]);
					return Promise.resolve();
				}

				updateScreenShareEncoding(...args) {
					calls.push(['update', args]);
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.throws(
			() => engine.publishScreenShare(1280, 720, 'h264', undefined, 30, false, undefined),
			/screen-share publish requires a non-empty captureId/,
		);
		assert.throws(
			() => engine.publishScreenShare(1280, 720, 'h264', undefined, 30, false, {captureId: ''}),
			/screen-share publish requires a non-empty captureId/,
		);
		assert.throws(
			() => engine.updateScreenShareEncoding(1280, 720, undefined, 30, undefined),
			/screen-share encoding update requires a non-empty captureId/,
		);
		assert.throws(
			() => engine.updateScreenShareEncoding(1280, 720, undefined, 30, {captureId: ''}),
			/screen-share encoding update requires a non-empty captureId/,
		);
		assert.deepEqual(calls, []);
	});

	test('delegates connect options through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				connect(url, token, e2eeKey, options) {
					calls.push({url, token, e2eeKey, options});
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		const key = Buffer.from('secret');
		await engine.connect('ws://localhost:7880', 'token', key, {
			autoSubscribe: false,
			adaptiveStream: true,
			dynacast: true,
		});
		assert.deepEqual(calls, [
			{
				url: 'ws://localhost:7880',
				token: 'token',
				e2eeKey: key,
				options: {
					autoSubscribe: false,
					adaptiveStream: true,
					dynacast: true,
				},
			},
		]);
	});

	test('delegates native screen frame sink handle creation through the VoiceEngine wrapper', () => {
		const calls = [];
		const handle = {native: true};
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				createScreenFrameSinkHandle(captureId) {
					calls.push(captureId);
					return handle;
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.equal(engine.createScreenFrameSinkHandle('capture-1'), handle);
		assert.deepEqual(calls, ['capture-1']);
	});

	test('treats native screen frame sink handles as unavailable for older voice bindings', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.equal(engine.createScreenFrameSinkHandle('capture-1'), null);
	});

	test('defaults dropped video callback metrics to zero for older native bindings', () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				inboundAudioFrames() {
					return 2;
				}

				inboundVideoFrames() {
					return 3;
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		assert.equal(engine.droppedVideoFrameCallbacks(), 0);
	});

	test('delegates remote track subscription updates through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				setRemoteTrackSubscription(participantIdentity, source, subscribed, enabled, quality) {
					calls.push({participantIdentity, source, subscribed, enabled, quality});
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.setRemoteTrackSubscription({
			participantIdentity: 'user_1_conn',
			source: 'screen_share',
			subscribed: true,
			enabled: false,
			quality: 'high',
		});

		assert.deepEqual(calls, [
			{
				participantIdentity: 'user_1_conn',
				source: 'screen_share',
				subscribed: true,
				enabled: false,
				quality: 'high',
			},
		]);
	});

	test('ignores remote track subscription updates for older native bindings', async () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.setRemoteTrackSubscription({
			participantIdentity: 'user_1_conn',
			source: 'camera',
			subscribed: true,
		});
	});

	test('delegates data packets through the VoiceEngine wrapper', async () => {
		const calls = [];
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishData(payload, reliable, topic, destinationIdentities) {
					calls.push({payload, reliable, topic, destinationIdentities});
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await engine.publishData(new Uint8Array([1, 2, 3]), {
			reliable: true,
			topic: 'screen-share-codec',
			destinationIdentities: ['user_1_conn'],
		});

		assert.equal(Buffer.isBuffer(calls[0].payload), true);
		assert.deepEqual([...calls[0].payload], [1, 2, 3]);
		assert.deepEqual(calls, [
			{
				payload: calls[0].payload,
				reliable: true,
				topic: 'screen-share-codec',
				destinationIdentities: ['user_1_conn'],
			},
		]);
	});

	test('rejects invalid data packet payloads in the wrapper', async () => {
		webrtcSender.__setBindingForTests({
			VoiceEngine: class {
				publishData() {
					return Promise.resolve();
				}
			},
		});

		const engine = new webrtcSender.VoiceEngine();
		await assert.rejects(() => engine.publishData('not-bytes'), /payload must be/);
	});

	test('returns an unavailable hardware encoder capability without a binding', () => {
		webrtcSender.__setBindingForTests(null);

		assert.deepEqual(webrtcSender.getHardwareEncoderCapability(), {
			available: false,
			backend: 'none',
			compiled: false,
			runtime: false,
			codecs: [],
			zeroCopy: false,
			nativeInputs: [],
			reason: 'native_binding_unavailable',
			detail: '@fluxer/webrtc-sender binding unavailable',
		});
	});

	test('exposes every VoiceEngine method declared in index.d.ts on the wrapper', async () => {
		const {readFileSync} = await import('node:fs');
		const dts = readFileSync(new URL('./index.d.ts', import.meta.url), 'utf8');
		const classStart = dts.indexOf('export declare class VoiceEngine {');
		assert.ok(classStart >= 0);
		const classEnd = dts.indexOf('\n}', classStart);
		assert.ok(classEnd > classStart);
		const classBody = dts.slice(classStart, classEnd);
		const declaredMethods = [...classBody.matchAll(/^\t([A-Za-z0-9_]+)\(/gm)]
			.map((match) => match[1])
			.filter((name) => name !== 'constructor');
		assert.ok(declaredMethods.length >= 30);
		const wrapperMethods = new Set(Object.getOwnPropertyNames(webrtcSender.VoiceEngine.prototype));
		const missing = declaredMethods.filter((name) => !wrapperMethods.has(name));
		assert.deepEqual(missing, []);
	});
});
