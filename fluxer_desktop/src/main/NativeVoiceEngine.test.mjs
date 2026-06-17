// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./NativeVoiceEngine.ts', import.meta.url));
const bridgePath = fileURLToPath(new URL('../../../packages/voice_engine_v2/src/bridge/index.ts', import.meta.url));
const bridgeFixturePath = fileURLToPath(
	new URL('../../../packages/voice_engine_v2/fixtures/bridge/bridge_contract.json', import.meta.url),
);
const nativeIpcRequestFixturePath = fileURLToPath(
	new URL('./fixtures/native_voice_engine_ipc_requests.json', import.meta.url),
);
const source = readFileSync(sourcePath, 'utf8');
const bridgeContractFixture = JSON.parse(readFileSync(bridgeFixturePath, 'utf8'));
const nativeIpcRequestFixture = JSON.parse(readFileSync(nativeIpcRequestFixturePath, 'utf8'));
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;
const bundledBridgeSource = esbuild.buildSync({
	entryPoints: [bridgePath],
	bundle: true,
	format: 'cjs',
	platform: 'node',
	target: 'node20',
	write: false,
}).outputFiles[0]?.text;
if (!bundledBridgeSource) {
	throw new Error('Failed to bundle voice engine v2 bridge for NativeVoiceEngine tests');
}
const bridgeModule = {exports: {}};
vm.runInContext(
	bundledBridgeSource,
	vm.createContext({
		exports: bridgeModule.exports,
		module: bridgeModule,
	}),
	{filename: bridgePath},
);
const voiceEngineBridge = bridgeModule.exports;
const {
	VOICE_ENGINE_V2_BRIDGE_VERSION,
	VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE,
	VOICE_ENGINE_V2_EVENT_CHANNELS,
	VOICE_ENGINE_V2_IPC_CHANNELS,
} = voiceEngineBridge;

function plain(value) {
	return structuredClone(value);
}

function materializeFixtureValue(value) {
	if (Array.isArray(value)) return value.map((item) => materializeFixtureValue(item));
	if (value && typeof value === 'object') {
		if (Array.isArray(value.$arrayBuffer)) {
			return Uint8Array.from(value.$arrayBuffer).buffer;
		}
		return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, materializeFixtureValue(item)]));
	}
	return value;
}

function makeSender() {
	const sender = new EventEmitter();
	sender.id = 100;
	sender.isDestroyed = () => false;
	sender.sent = [];
	sender.send = (...args) => {
		sender.sent.push(args);
	};
	return sender;
}

async function flushAsyncWork() {
	await new Promise((resolve) => setImmediate(resolve));
}

function loadNativeVoiceEngine({hardwareCapabilities, connectHandler, nativeModuleOverrides} = {}) {
	const handlers = new Map();
	const calls = {
		acquire: [],
		release: 0,
		prewarm: 0,
		assertEngineBridgeVersion: [],
		logs: {info: [], warn: [], error: []},
	};
	const engineInstances = [];
	const webContentsList = [];

	class MockVoiceEngine {
		constructor() {
			this.ops = [];
			this.calls = {
				connect: [],
				disconnect: 0,
				publishScreenShare: [],
				updateScreenShareEncoding: [],
				unpublishScreenShare: 0,
				publishScreenShareAudio: [],
				screenSharePcm: [],
				screenShareFloat: [],
				unpublishScreenShareAudio: 0,
				remoteTrackSubscription: [],
				createScreenFrameSinkHandle: [],
				publishDeviceScreenShare: [],
				publishCamera: [],
				publishNativeCameraSink: [],
				publishProcessedCamera: [],
				processedCameraFrames: [],
				cameraBackgroundFrames: [],
				clearCameraBackgroundFrame: 0,
				updateCameraCapture: [],
				publishData: [],
				setParticipantVolume: [],
				startCameraPreview: [],
				stopCameraPreview: 0,
				clearVideoFrameCallback: 0,
			};
			engineInstances.push(this);
		}

		async connect(...args) {
			this.calls.connect.push(args);
			this.ops.push(['connect', args[0]]);
			if (connectHandler) {
				await connectHandler(this, ...args);
			}
		}

		async disconnect() {
			this.calls.disconnect += 1;
			this.ops.push(['disconnect']);
		}

		isConnected() {
			return true;
		}

		async publishScreenShare(...args) {
			this.calls.publishScreenShare.push(args);
		}

		async updateScreenShareEncoding(...args) {
			this.calls.updateScreenShareEncoding.push(args);
		}

		createScreenFrameSinkHandle(captureId) {
			this.calls.createScreenFrameSinkHandle.push(captureId);
			return {native: true, captureId};
		}

		async unpublishScreenShare() {
			this.calls.unpublishScreenShare += 1;
		}

		isPublishingScreen() {
			return true;
		}

		async publishScreenShareAudio(...args) {
			this.calls.publishScreenShareAudio.push(args);
		}

		async pushScreenSharePcm(...args) {
			this.calls.screenSharePcm.push(args);
			return true;
		}

		async pushScreenShareFloat(...args) {
			this.calls.screenShareFloat.push(args);
			return true;
		}

		async unpublishScreenShareAudio() {
			this.calls.unpublishScreenShareAudio += 1;
		}

		isPublishingScreenAudio() {
			return false;
		}

		async publishMicrophone() {}

		async pushPcm() {
			return true;
		}

		async setMicEnabled() {}

		listAudioOutputDevices() {
			return [];
		}

		async setAudioOutputDevice() {}

		async setParticipantVolume(...args) {
			this.calls.setParticipantVolume.push(args);
		}

		async setRemoteTrackSubscription(...args) {
			this.calls.remoteTrackSubscription.push(args);
		}

		async publishData(...args) {
			this.calls.publishData.push(args);
		}

		listCameraDevices() {
			return [];
		}

		async publishCamera(...args) {
			this.calls.publishCamera.push(args);
		}

		async publishNativeCameraSink(...args) {
			this.calls.publishNativeCameraSink.push(args);
			return {trackSid: 'TR_native_camera'};
		}

		async publishProcessedCamera(...args) {
			this.calls.publishProcessedCamera.push(args);
			return {trackSid: 'TR_processed_camera'};
		}

		async pushProcessedCameraFrame(...args) {
			this.calls.processedCameraFrames.push(args);
			return true;
		}

		async pushCameraBackgroundFrame(...args) {
			this.calls.cameraBackgroundFrames.push(args);
			return true;
		}

		clearCameraBackgroundFrame() {
			this.calls.clearCameraBackgroundFrame += 1;
		}

		async updateCameraCapture(...args) {
			this.calls.updateCameraCapture.push(args);
		}

		async publishDeviceScreenShare(...args) {
			this.calls.publishDeviceScreenShare.push(args);
		}

		async unpublishCamera() {}

		async startCameraPreview(...args) {
			this.calls.startCameraPreview.push(args);
			return {trackSid: 'local-camera-preview', width: 1280, height: 720, frameRate: 30};
		}

		stopCameraPreview() {
			this.calls.stopCameraPreview += 1;
		}

		async getConnectionStats() {
			return {rttMs: null, outbound: [], inbound: []};
		}

		setEventCallback(callback) {
			this.eventCallback = callback;
		}

		setVideoFrameCallback(callback) {
			this.videoFrameCallback = callback;
		}

		clearVideoFrameCallback() {
			this.calls.clearVideoFrameCallback += 1;
			this.videoFrameCallback = null;
		}
	}

	const nativeModule = {
		isSupported: () => true,
		getEngineBridgeVersion: () => VOICE_ENGINE_V2_BRIDGE_VERSION,
		assertEngineBridgeVersion: (version) => {
			calls.assertEngineBridgeVersion.push(version);
		},
		getCapabilities: () => ({
			microphoneCapture: true,
			syntheticMicrophonePcm: true,
			cameraCapture: true,
			nativeCameraBackgrounds: false,
			screenShare: true,
			screenShareAudio: true,
			deviceLists: true,
			outputDeviceSelection: true,
			participantVolume: true,
			remoteTrackSubscription: true,
			dataChannel: true,
			connectionStats: true,
			nativeVideoFrames: true,
			hardwareEncoderCapabilities: true,
		}),
		prewarmVoiceEngine: () => {
			calls.prewarm += 1;
		},
		getHardwareEncoderCapabilities: () =>
			hardwareCapabilities ?? {
				available: true,
				backend: 'nvenc',
				compiled: true,
				runtime: true,
				codecs: ['hevc'],
				zeroCopy: true,
				nativeInputs: ['dmabuf'],
			},
		VoiceEngine: MockVoiceEngine,
		loadError: null,
		...nativeModuleOverrides,
	};

	function requireStub(specifier) {
		if (specifier === 'node:assert/strict') {
			return assert;
		}
		if (specifier === 'node:module') {
			return {
				createRequire: () => (moduleSpecifier) => {
					if (moduleSpecifier === '@fluxer/webrtc-sender') return nativeModule;
					throw new Error(`Unexpected createRequire import: ${moduleSpecifier}`);
				},
			};
		}
		if (specifier === '@electron/common/Logger') {
			return {
				createChildLogger: () => ({
					info: (...args) => calls.logs.info.push(args),
					warn: (...args) => calls.logs.warn.push(args),
					error: (...args) => calls.logs.error.push(args),
				}),
			};
		}
		if (specifier === '@fluxer/voice_engine_v2/bridge') {
			return voiceEngineBridge;
		}
		if (specifier === 'electron') {
			return {
				ipcMain: {
					handle(channel, handler) {
						handlers.set(channel, handler);
					},
					removeHandler(channel) {
						handlers.delete(channel);
					},
				},
				webContents: {
					getAllWebContents() {
						return webContentsList.slice();
					},
				},
			};
		}
		if (specifier === './NativeVoiceEngineIpcCore') {
			class NativeVoiceEngineCapabilityError extends Error {
				constructor(capability, message) {
					super(message);
					this.capability = capability;
				}
			}
			class NativeVoiceEngineNotConnectedError extends Error {}
			class NativeVoiceEngineInvalidArgsError extends Error {}
			return {
				handleNativeVoiceEngineListAudioInputDevices: () => [],
				handleNativeVoiceEngineListAudioOutputDevices: () => [],
				handleNativeVoiceEngineListCameraDevices: async (engine) => {
					if (!engine) throw new Error('Native voice engine is unavailable');
					return engine.listCameraDevices();
				},
				handleNativeVoiceEnginePublishMicrophone: async () => {},
				NativeVoiceEngineCapabilityError,
				NativeVoiceEngineNotConnectedError,
				NativeVoiceEngineInvalidArgsError,
			};
		}
		if (specifier === './StreamingPriority') {
			return {
				acquireStreamingPriority(sender) {
					calls.acquire.push(sender);
				},
				releaseStreamingPriority() {
					calls.release += 1;
				},
			};
		}
		throw new Error(`Unexpected import: ${specifier}`);
	}

	const module = {exports: {}};
	const context = vm.createContext({
		ArrayBuffer,
		Buffer,
		clearTimeout,
		console,
		exports: module.exports,
		module,
		process: {env: {}, platform: 'linux'},
		require: requireStub,
		setTimeout,
		Uint8Array,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});

	return {calls, engineInstances, handlers, module: module.exports, webContentsList};
}

async function applyFixtureSetup(harness, setup) {
	if (!setup) return makeSender();
	const sender = makeSender();
	await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
		{sender},
		{url: 'wss://voice.example.test/setup', token: 'setup-token'},
	);
	if (setup === 'screen') {
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen)(null, {
			captureId: 'fixture-screen',
			width: 1920,
			height: 1080,
			codec: 'h264',
			maxFramerate: 60,
			zeroCopyRequired: true,
		});
	}
	return sender;
}

function latestFixtureEngine(harness) {
	return harness.engineInstances[harness.engineInstances.length - 1];
}

describe('NativeVoiceEngine v2 bridge contract fixtures', () => {
	test('registers and removes every v2 IPC channel from the bridge contract fixture', () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		assert.equal(bridgeContractFixture.version, VOICE_ENGINE_V2_BRIDGE_VERSION);
		assert.deepEqual(bridgeContractFixture.eventChannels, plain(VOICE_ENGINE_V2_EVENT_CHANNELS));
		assert.deepEqual(
			Array.from(harness.handlers.keys()).sort(),
			Object.values(bridgeContractFixture.ipcChannels).sort(),
		);

		harness.module.cleanupNativeVoiceEngine();

		assert.equal(harness.handlers.size, 0);
	});
});

describe('NativeVoiceEngine bridge version pair assertion', () => {
	test('asserts the host bridge version against the native addon at load time', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		const supported = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isSupported)();

		assert.equal(supported, true);
		assert.deepEqual(harness.calls.assertEngineBridgeVersion, [VOICE_ENGINE_V2_BRIDGE_VERSION]);
		assert.deepEqual(harness.calls.logs.error, []);
	});

	test('disables the native voice engine when the addon reports a mismatched bridge version', async () => {
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {getEngineBridgeVersion: () => VOICE_ENGINE_V2_BRIDGE_VERSION + 1},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		const supported = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isSupported)();
		const capabilities = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getCapabilities)();
		const hardware = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getHardwareEncoderCapabilities)();

		assert.equal(supported, false);
		assert.equal(capabilities.microphoneCapture, false);
		assert.equal(capabilities.screenShare, false);
		assert.equal(hardware.available, false);
		assert.equal(hardware.reason, 'load-failed');
		assert.match(hardware.detail, /bridge version mismatch/);
		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
					{sender: makeSender()},
					{url: 'wss://livekit.invalid', token: 'token'},
				),
			/Native voice engine unavailable/,
		);
		assert.equal(harness.calls.logs.error.length, 1);
		assert.match(harness.calls.logs.error[0][1].detail, /bridge version mismatch/);
		assert.equal(harness.calls.logs.error[0][1].hostBridgeVersion, VOICE_ENGINE_V2_BRIDGE_VERSION);
		assert.deepEqual(harness.calls.assertEngineBridgeVersion, []);
		assert.equal(harness.engineInstances.length, 0);
	});

	test('disables the native voice engine when the addon lacks bridge version exports', async () => {
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {getEngineBridgeVersion: undefined, assertEngineBridgeVersion: undefined},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		const supported = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isSupported)();

		assert.equal(supported, false);
		assert.equal(harness.calls.logs.error.length, 1);
		assert.match(harness.calls.logs.error[0][1].detail, /does not export getEngineBridgeVersion/);
	});

	test('disables the native voice engine when the addon rejects the host bridge version', async () => {
		const hostVersion = VOICE_ENGINE_V2_BRIDGE_VERSION;
		const addonVersion = VOICE_ENGINE_V2_BRIDGE_VERSION + 1;
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				assertEngineBridgeVersion: () => {
					throw new Error(
						`voice engine bridge version mismatch: host sent ${hostVersion}, native addon expects ${addonVersion}`,
					);
				},
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		const supported = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isSupported)();

		assert.equal(supported, false);
		assert.equal(harness.calls.logs.error.length, 1);
		assert.match(harness.calls.logs.error[0][1].detail, new RegExp(`rejected bridge version ${hostVersion}\\b`));
		assert.match(harness.calls.logs.error[0][1].detail, new RegExp(`native addon expects ${addonVersion}\\b`));
	});
});

describe('NativeVoiceEngine v2 IPC request fixtures', () => {
	for (const fixtureCase of nativeIpcRequestFixture.cases) {
		test(`replays fixture: ${fixtureCase.name}`, async () => {
			const harness = loadNativeVoiceEngine();
			harness.module.registerNativeVoiceEngineHandlers();
			const sender = await applyFixtureSetup(harness, fixtureCase.setup);
			const channel = VOICE_ENGINE_V2_IPC_CHANNELS[fixtureCase.channel];
			const handler = harness.handlers.get(channel);
			assert.equal(typeof handler, 'function', `${fixtureCase.name} references an unregistered channel`);
			const event = fixtureCase.event === 'sender' ? {sender} : null;
			const args = materializeFixtureValue(fixtureCase.args ?? []);
			const invoke = () => handler(event, ...args);

			if (fixtureCase.expected.throws) {
				await assert.rejects(invoke, new RegExp(fixtureCase.expected.throws));
				return;
			}

			const result = await invoke();
			if (fixtureCase.expected.result === 'void') {
				assert.equal(result, undefined);
			} else {
				assert.deepEqual(plain(result), fixtureCase.expected.result);
			}

			if (fixtureCase.expected.engineCalls) {
				const engine = latestFixtureEngine(harness);
				assert.ok(engine, `${fixtureCase.name} expected a native engine instance`);
				for (const [callName, callCount] of Object.entries(fixtureCase.expected.engineCalls)) {
					const calls = engine.calls[callName];
					assert.equal(Array.isArray(calls) ? calls.length : calls, callCount, `${fixtureCase.name} ${callName}`);
				}
			}
		});
	}
});

describe('NativeVoiceEngine connection lifecycle', () => {
	test('constructs the singleton and prewarms the native voice backend once when requested', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		assert.equal(harness.calls.prewarm, 0);
		assert.equal(harness.engineInstances.length, 0);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		assert.equal(harness.calls.prewarm, 1);
		assert.equal(harness.engineInstances.length, 1);
	});

	test('connect uses the prewarmed singleton instead of constructing an engine per call', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		await connect({sender}, {url: 'wss://livekit.invalid/prewarmed', token: 'prewarmed-token'});

		assert.equal(harness.engineInstances.length, 1);
		assert.deepEqual(harness.engineInstances[0].calls.connect[0].slice(0, 2), [
			'wss://livekit.invalid/prewarmed',
			'prewarmed-token',
		]);
	});

	test('connect waits for a pending prewarm before dialing', async () => {
		let resolvePrewarm;
		const prewarmPending = new Promise((resolve) => {
			resolvePrewarm = resolve;
		});
		const pendingConnects = [];
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				prewarmVoiceEngine: () => {
					harness.calls.prewarm += 1;
					return prewarmPending;
				},
			},
			connectHandler: (engine) =>
				new Promise((resolve) => {
					pendingConnects.push({engine, resolve});
				}),
		});
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);

		const connectPromise = connect(
			{sender: makeSender()},
			{url: 'wss://livekit.invalid/cold-start', token: 'cold-start-token'},
		);
		await flushAsyncWork();

		assert.equal(harness.calls.prewarm, 1);
		assert.equal(pendingConnects.length, 0);
		resolvePrewarm();
		await flushAsyncWork();
		assert.equal(pendingConnects.length, 1);
		pendingConnects[0].resolve();
		await connectPromise;
	});

	test('prewarm retries transient addon failures before succeeding', async () => {
		let attempts = 0;
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				prewarmVoiceEngine: () => {
					attempts += 1;
					if (attempts < 3) {
						throw new Error('list audio output devices: ADM reported negative audio device count: -1');
					}
				},
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		assert.equal(attempts, 3);
		const readiness = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();
		assert.deepEqual(plain(readiness), {ready: true});
	});

	test('set-audio-output-device retries transient negative device-count failures', async () => {
		let setAttempts = 0;
		class FlakyAdmVoiceEngine {
			async setAudioOutputDevice(deviceId) {
				setAttempts += 1;
				if (setAttempts < 3) {
					throw new Error('list audio output devices: ADM reported negative audio device count: -1');
				}
				this.lastDeviceId = deviceId;
			}
		}
		const engineInstances = [];
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				VoiceEngine: class extends FlakyAdmVoiceEngine {
					constructor() {
						super();
						engineInstances.push(this);
					}
				},
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setAudioOutputDevice)({}, 'default');
		await flushAsyncWork();

		assert.equal(setAttempts, 3);
		assert.equal(engineInstances.length, 1);
		assert.equal(engineInstances[0].lastDeviceId, 'default');
	});

	test('set-audio-output-device does not retry non-transient failures', async () => {
		let setAttempts = 0;
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				VoiceEngine: class {
					async setAudioOutputDevice() {
						setAttempts += 1;
						throw new Error('set audio output device: device not found');
					}
				},
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		await assert.rejects(
			harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setAudioOutputDevice)({}, 'missing-guid'),
			/device not found/,
		);

		assert.equal(setAttempts, 1);
	});

	test('reuses the singleton across disconnect and reconnect cycles', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const disconnect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect);

		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/first', token: 'first-token'});
		const firstEngine = harness.engineInstances[0];

		await disconnect();
		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/second', token: 'second-token'});

		assert.equal(harness.engineInstances.length, 1);
		assert.deepEqual(firstEngine.calls.connect[1].slice(0, 2), ['wss://livekit.invalid/second', 'second-token']);
		assert.deepEqual(firstEngine.ops, [
			['connect', 'wss://livekit.invalid/first'],
			['disconnect'],
			['connect', 'wss://livekit.invalid/second'],
		]);
	});

	test('a second connect awaits the disconnect of the previous session before dialing', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);

		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/first', token: 'first-token'});
		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/second', token: 'second-token'});

		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];
		assert.deepEqual(engine.ops, [
			['connect', 'wss://livekit.invalid/first'],
			['disconnect'],
			['connect', 'wss://livekit.invalid/second'],
		]);
	});

	test('a connect superseded before dialing rejects while latest replacement dials', async () => {
		const pendingConnects = [];
		const harness = loadNativeVoiceEngine({
			connectHandler: (engine) =>
				new Promise((resolve) => {
					pendingConnects.push({engine, resolve});
				}),
		});
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		const firstConnect = connect({sender}, {url: 'wss://livekit.invalid/first', token: 'first-token'});
		await flushAsyncWork();
		assert.equal(pendingConnects.length, 1);

		const secondConnect = connect({sender}, {url: 'wss://livekit.invalid/second', token: 'second-token'});
		const secondRejected = assert.rejects(secondConnect, /connect superseded/);
		const thirdConnect = connect({sender}, {url: 'wss://livekit.invalid/third', token: 'third-token'});
		await flushAsyncWork();
		assert.equal(pendingConnects.length, 2);
		await secondRejected;

		pendingConnects[0].resolve();
		await assert.rejects(firstConnect, /session was replaced/);
		pendingConnects[1].resolve();
		await thirdConnect;

		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];
		const dialedUrls = engine.calls.connect.map((args) => args[0]);
		assert.deepEqual(dialedUrls, ['wss://livekit.invalid/first', 'wss://livekit.invalid/third']);
	});

	test('clears the native video-frame callback when a session is torn down', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const disconnect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect);

		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/clear', token: 'clear-token'});
		const engine = harness.engineInstances[0];
		assert.equal(typeof engine.videoFrameCallback, 'function');

		await disconnect();

		assert.ok(engine.calls.clearVideoFrameCallback >= 1);
		assert.equal(engine.videoFrameCallback, null);
	});

	test('tears down the active session when the owner renderer process exits', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		await connect({sender}, {url: 'wss://livekit.invalid/renderer-gone', token: 'renderer-token'});
		const engine = harness.engineInstances[0];

		sender.emit('render-process-gone', {}, {reason: 'crashed'});
		await flushAsyncWork();

		assert.equal(engine.calls.disconnect, 1);
		assert.equal(await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isConnected)(), false);
	});

	test('tears down the active session on owner main-frame reload navigation', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		await connect({sender}, {url: 'wss://livekit.invalid/reload', token: 'reload-token'});
		const engine = harness.engineInstances[0];

		sender.emit('did-start-navigation', {}, 'app://fluxer/reload', false, true);
		await flushAsyncWork();

		assert.equal(engine.calls.disconnect, 1);
	});

	test('keeps the active session for same-document and subframe navigation', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		await connect({sender}, {url: 'wss://livekit.invalid/navigation', token: 'navigation-token'});
		const engine = harness.engineInstances[0];

		sender.emit('did-start-navigation', {}, 'app://fluxer/channel', true, true);
		sender.emit('did-start-navigation', {}, 'https://frame.invalid', false, false);
		await flushAsyncWork();

		assert.equal(engine.calls.disconnect, 0);
		assert.equal(await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.isConnected)(), true);
	});

	test('falls back to a no-op video-frame callback for engines without a clear export', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const disconnect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect);

		await connect({sender: makeSender()}, {url: 'wss://livekit.invalid/legacy', token: 'legacy-token'});
		const engine = harness.engineInstances[0];
		engine.clearVideoFrameCallback = undefined;

		await disconnect();

		assert.equal(typeof engine.videoFrameCallback, 'function');
		assert.equal(engine.videoFrameCallback('{}', Buffer.alloc(0)), undefined);
	});

	test('does not forward native events after disconnecting a pending connect', async () => {
		const pendingConnects = [];
		const harness = loadNativeVoiceEngine({
			connectHandler: (engine) =>
				new Promise((resolve) => {
					pendingConnects.push({engine, resolve});
				}),
		});
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const disconnect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect);
		const sender = makeSender();

		const connectPromise = connect({sender}, {url: 'wss://livekit.invalid/slow', token: 'slow-token'});
		await flushAsyncWork();
		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];

		await disconnect();
		engine.eventCallback('participantJoined', '{"identity":"user_1_old-connection"}');

		assert.deepEqual(sender.sent, []);

		pendingConnects[0].resolve();
		await assert.rejects(connectPromise, /session was replaced/);
	});

	test('starts replacement connect before stale native connect completion settles', async () => {
		const pendingConnects = [];
		const harness = loadNativeVoiceEngine({
			connectHandler: (engine) =>
				new Promise((resolve) => {
					pendingConnects.push({engine, resolve});
				}),
		});
		harness.module.registerNativeVoiceEngineHandlers();
		const connect = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect);
		const sender = makeSender();

		const firstConnect = connect({sender}, {url: 'wss://livekit.invalid/first', token: 'first-token'});
		await flushAsyncWork();
		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];

		const secondConnect = connect({sender}, {url: 'wss://livekit.invalid/second', token: 'second-token'});
		await flushAsyncWork();
		assert.equal(harness.engineInstances.length, 1);
		assert.equal(engine.calls.disconnect, 1);
		assert.equal(pendingConnects.length, 2);

		pendingConnects[0].resolve();
		await assert.rejects(firstConnect, /session was replaced/);
		assert.equal(engine.calls.disconnect, 1);

		pendingConnects[1].resolve();
		await secondConnect;
		assert.equal(engine.calls.disconnect, 1);

		assert.equal(harness.engineInstances.length, 1);
		assert.deepEqual(
			engine.calls.connect.map((args) => args[0]),
			['wss://livekit.invalid/first', 'wss://livekit.invalid/second'],
		);
	});
});

async function publishScreen(harness, captureId = 'screen-1') {
	harness.module.registerNativeVoiceEngineHandlers();
	const sender = makeSender();
	await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
		{sender},
		{url: 'wss://livekit.invalid', token: 'token'},
	);
	await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen)(null, {
		captureId,
		width: 1920,
		height: 1080,
		codec: 'h265',
		maxBitrateBps: 12_000_000,
		maxFramerate: 60,
	});
	return {engine: harness.engineInstances[0], sender};
}

describe('NativeVoiceEngine screen frame routing', () => {
	test('publishes screen share with source pacing and exposes a native frame sink handle', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen)(null, {
			captureId: 'display-capture',
			width: 1920,
			height: 1080,
			codec: 'h265',
			maxBitrateBps: 12_000_000,
			maxFramerate: 60,
			pacing: 'source',
			trackName: 'display-capture',
		});

		const engine = harness.engineInstances[0];
		assert.equal(engine.calls.publishScreenShare[0][6].pacing, 'source');
		assert.equal(engine.calls.publishScreenShare[0][6].trackName, 'display-capture');
		const handle = harness.module.createNativeVoiceEngineScreenFrameSinkHandle('display-capture');
		assert.deepEqual(handle, {native: true, captureId: 'display-capture'});
		assert.equal(harness.module.createNativeVoiceEngineScreenFrameSinkHandle('other-capture'), null);
		assert.deepEqual(engine.calls.createScreenFrameSinkHandle, ['display-capture']);
		assert.deepEqual(harness.calls.acquire, [sender]);
	});

	test('forwards device screen share publish options to the native engine', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishDeviceScreenShare)(null, {
			deviceId: 'studio-display-camera',
			width: 1280,
			height: 720,
			frameRate: 30,
			codec: 'h264',
			maxBitrateBps: 4_000_000,
			maxFramerate: 30,
		});

		const engine = harness.engineInstances[0];
		assert.deepEqual(engine.calls.publishDeviceScreenShare, [
			[
				{
					deviceId: 'studio-display-camera',
					width: 1280,
					height: 720,
					frameRate: 30,
					codec: 'h264',
					maxBitrateBps: 4_000_000,
					maxFramerate: 30,
				},
			],
		]);
	});

	test('keeps screen-share audio published when screen video is unpublished', async () => {
		const harness = loadNativeVoiceEngine();
		const {engine} = await publishScreen(harness, 'display-capture');

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreenAudio)(null, {
			sampleRate: 48000,
			numChannels: 2,
		});
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.unpublishScreen)(null);
		const pushed = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushScreenAudioPcm)(null, {
			sampleRate: 48000,
			numChannels: 2,
			samples: new Uint8Array([1, 2, 3, 4]).buffer,
		});

		assert.equal(engine.calls.publishScreenShareAudio.length, 1);
		assert.deepEqual(engine.calls.publishScreenShareAudio[0], [48000, 2]);
		assert.equal(engine.calls.unpublishScreenShare, 1);
		assert.equal(engine.calls.unpublishScreenShareAudio, 0);
		assert.equal(pushed, true);
		assert.equal(engine.calls.screenSharePcm.length, 1);
		assert.equal(Buffer.isBuffer(engine.calls.screenSharePcm[0][0]), true);
		assert.deepEqual(engine.calls.screenSharePcm[0].slice(1), [48000, 2]);
		assert.equal(harness.calls.release, 1);
	});

	test('pushes screen-share float audio using the typed-array view length', async () => {
		const harness = loadNativeVoiceEngine();
		const {engine} = await publishScreen(harness, 'display-capture');

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreenAudio)(null, {
			sampleRate: 48000,
			numChannels: 1,
		});
		const backing = new Float32Array([99, 0.25, -0.25, 88]);
		const view = backing.subarray(1, 3);
		const pushed = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushScreenAudioFloat)(null, {
			sampleRate: 48000,
			numChannels: 1,
			samples: view,
		});

		assert.equal(pushed, true);
		assert.equal(engine.calls.screenShareFloat.length, 1);
		assert.equal(Buffer.isBuffer(engine.calls.screenShareFloat[0][0]), true);
		assert.equal(engine.calls.screenShareFloat[0][0].byteLength, view.byteLength);
		assert.equal(engine.calls.screenShareFloat[0][0].readFloatLE(0), 0.25);
		assert.equal(engine.calls.screenShareFloat[0][0].readFloatLE(Float32Array.BYTES_PER_ELEMENT), -0.25);
		assert.deepEqual(engine.calls.screenShareFloat[0].slice(1), [48000, 1]);
	});

	test('replaces screen captures without releasing screen-share audio or duplicating priority', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen)(null, {
			captureId: 'display-capture',
			width: 2560,
			height: 1440,
			codec: 'h264',
			maxFramerate: 60,
		});
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreenAudio)(null, {
			sampleRate: 48000,
			numChannels: 2,
		});
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen)(null, {
			captureId: 'window-capture',
			width: 1280,
			height: 720,
			codec: 'h265',
			maxBitrateBps: 8_000_000,
			maxFramerate: 30,
		});

		const engine = harness.engineInstances[0];
		assert.equal(engine.calls.publishScreenShare.length, 2);
		assert.deepEqual(engine.calls.publishScreenShare[0].slice(0, 5), [2560, 1440, 'h264', undefined, 60]);
		assert.deepEqual(engine.calls.publishScreenShare[1].slice(0, 5), [1280, 720, 'h265', 8_000_000, 30]);
		assert.equal(engine.calls.unpublishScreenShareAudio, 0);
		assert.deepEqual(harness.calls.acquire, [sender]);
		assert.equal(harness.calls.release, 0);
		assert.deepEqual(harness.module.createNativeVoiceEngineScreenFrameSinkHandle('display-capture'), null);
		assert.deepEqual(harness.module.createNativeVoiceEngineScreenFrameSinkHandle('window-capture'), {
			native: true,
			captureId: 'window-capture',
		});
	});

	test('updates active screen-share encoding over IPC', async () => {
		const harness = loadNativeVoiceEngine();
		const {engine} = await publishScreen(harness, 'display-capture');

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.updateScreenShareEncoding)(null, {
			captureId: 'display-capture',
			width: 1280,
			height: 720,
			frameRate: 30,
			maxBitrateBps: 3_000_000,
		});
		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.updateScreenShareEncoding)(null, {
					captureId: 'other-capture',
					width: 854,
					height: 480,
				}),
			/capture id mismatch/,
		);

		assert.deepEqual(plain(engine.calls.updateScreenShareEncoding), [
			[1280, 720, 3_000_000, 30, {captureId: 'display-capture'}],
		]);
		assert.equal(engine.calls.publishScreenShare.length, 1);
		assert.equal(engine.calls.unpublishScreenShare, 0);
	});

	test('rejects screen-share encoding updates that change codec or hardware mode', async () => {
		const harness = loadNativeVoiceEngine();
		const {engine} = await publishScreen(harness, 'display-capture');

		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.updateScreenShareEncoding)(null, {
					captureId: 'display-capture',
					width: 1920,
					height: 1080,
					codec: 'h265',
					hardwareEncoding: true,
					maxBitrateBps: 6_000_000,
				}),
			/cannot change publication codec or hardware mode/,
		);

		assert.equal(engine.calls.updateScreenShareEncoding.length, 0);
		assert.equal(engine.calls.publishScreenShare.length, 1);
	});

	test('forwards watch and unwatch operations with source, enablement, and quality intact', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender: makeSender()},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setRemoteTrackSubscription)(null, {
			participantIdentity: 'alice',
			source: 'screen_share',
			subscribed: true,
			quality: 'high',
		});
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setRemoteTrackSubscription)(null, {
			participantIdentity: 'alice',
			source: 'screen_share_audio',
			subscribed: false,
		});
		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setRemoteTrackSubscription)(null, {
					participantIdentity: 'alice',
					source: 'screen_share',
					subscribed: true,
					quality: 'ultra',
				}),
			/Invalid voice-engine set-remote-track-subscription args/,
		);

		assert.deepEqual(plain(harness.engineInstances[0].calls.remoteTrackSubscription), [
			[
				{
					participantIdentity: 'alice',
					source: 'screen_share',
					subscribed: true,
					enabled: true,
					quality: 'high',
				},
			],
			[
				{
					participantIdentity: 'alice',
					source: 'screen_share_audio',
					subscribed: false,
					enabled: false,
				},
			],
		]);
	});

	test('reports VideoToolbox hardware capability from the native addon', async () => {
		const harness = loadNativeVoiceEngine({
			hardwareCapabilities: {
				available: true,
				backend: 'videotoolbox',
				compiled: true,
				runtime: true,
				codecs: ['h264', 'h265', 42],
				zeroCopy: false,
				nativeInputs: ['cvpixelbuffer', 42],
				detail: 'macOS hardware encoder available',
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		const capabilities = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getHardwareEncoderCapabilities)();

		assert.deepEqual(plain(capabilities), {
			available: true,
			backend: 'videotoolbox',
			compiled: true,
			runtime: true,
			codecs: ['h264', 'h265'],
			zeroCopy: false,
			nativeInputs: ['cvpixelbuffer'],
			detail: 'macOS hardware encoder available',
		});
	});

	test('reports native voice engine capabilities from the native addon', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		const capabilities = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getCapabilities)();

		assert.equal(capabilities.microphoneCapture, true);
		assert.equal(capabilities.screenShare, true);
		assert.equal(capabilities.connectionStats, true);
	});

	test('maps native addon microphone not-connected errors to the typed operation code', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender: makeSender()},
			{url: 'wss://livekit.invalid', token: 'token'},
		);
		harness.engineInstances[0].setMicEnabled = async () => {
			throw new Error('not connected');
		};

		const result = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setMicEnabled)(null, true);

		assert.deepEqual(plain(result), {
			ok: false,
			error: {
				code: 'not-connected',
				message: 'not connected',
				capability: 'microphoneCapture',
			},
		});
	});

	test('rejects device microphone operations while the audio device module is warming', async () => {
		let resolveProbe;
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				probeAudioDeviceModule: () =>
					new Promise((resolve) => {
						resolveProbe = () => resolve(true);
					}),
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender: makeSender()},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		const publishResult = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.publishMicrophone)(null, {
			deviceId: 'default',
		});
		const enableResult = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.setMicEnabled)(null, true);

		assert.deepEqual(plain(publishResult), {
			ok: false,
			error: {
				code: 'native-error',
				message: 'Native audio device module is warming',
				capability: 'microphoneCapture',
			},
		});
		assert.deepEqual(plain(enableResult), {
			ok: false,
			error: {
				code: 'native-error',
				message: 'Native audio device module is warming',
				capability: 'microphoneCapture',
			},
		});
		assert.equal(typeof resolveProbe, 'function');
		resolveProbe();
		await flushAsyncWork();
	});
});

describe('NativeVoiceEngine camera preview sessions', () => {
	test('starts a standalone preview engine when not connected and stops it on stop', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();

		const result = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)(
			{sender},
			{backgroundMode: 'blur'},
		);

		assert.deepEqual(plain(result), {trackSid: 'local-camera-preview', width: 1280, height: 720, frameRate: 30});
		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];
		assert.deepEqual(engine.calls.startCameraPreview, [[{backgroundMode: 'blur'}]]);
		assert.equal(typeof engine.videoFrameCallback, 'function');

		const frameBuffer = Buffer.from([1, 2, 3]);
		engine.videoFrameCallback(JSON.stringify({trackSid: 'local-camera-preview'}), frameBuffer);
		assert.equal(sender.sent.length, 1);
		assert.equal(sender.sent[0][0], VOICE_ENGINE_V2_EVENT_CHANNELS.videoFrame);
		assert.equal(sender.sent[0][1].data, frameBuffer);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.stopCameraPreview)();
		assert.equal(engine.calls.stopCameraPreview, 1);
		assert.equal(engine.calls.clearVideoFrameCallback, 1);
	});

	test('routes camera preview through the active session engine when connected', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://voice.example.test/preview', token: 'preview-token'},
		);

		const result = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});

		assert.equal(harness.engineInstances.length, 1);
		assert.equal(result.trackSid, 'local-camera-preview');
		assert.deepEqual(harness.engineInstances[0].calls.startCameraPreview, [[{}]]);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.stopCameraPreview)();
		assert.equal(harness.engineInstances[0].calls.stopCameraPreview, 1);
	});

	test('stops a standalone preview on the singleton when a call connects', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});
		assert.equal(harness.engineInstances.length, 1);
		const engine = harness.engineInstances[0];

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://voice.example.test/after-preview', token: 'after-preview-token'},
		);

		assert.equal(harness.engineInstances.length, 1);
		assert.equal(engine.calls.stopCameraPreview, 1);
		assert.equal(engine.calls.clearVideoFrameCallback, 1);
		assert.deepEqual(engine.calls.connect[0].slice(0, 2), [
			'wss://voice.example.test/after-preview',
			'after-preview-token',
		]);
	});

	test('routes a preview after a call ends through the same singleton engine', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://voice.example.test/call', token: 'call-token'},
		);
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect)();

		const result = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});

		assert.equal(harness.engineInstances.length, 1);
		assert.equal(result.trackSid, 'local-camera-preview');
		const engine = harness.engineInstances[0];
		assert.deepEqual(engine.calls.startCameraPreview, [[{}]]);

		engine.videoFrameCallback(JSON.stringify({trackSid: 'local-camera-preview'}), Buffer.from([1, 2, 3]));
		assert.equal(sender.sent.length, 1);
		assert.equal(sender.sent[0][0], VOICE_ENGINE_V2_EVENT_CHANNELS.videoFrame);
	});

	test('stops a standalone preview when the owner renderer process exits', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});
		const engine = harness.engineInstances[0];

		sender.emit('render-process-gone', {}, {reason: 'crashed'});
		await flushAsyncWork();

		assert.equal(engine.calls.stopCameraPreview, 1);
		assert.equal(engine.calls.clearVideoFrameCallback, 1);
	});

	test('stops a standalone preview on owner main-frame reload navigation', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});
		const engine = harness.engineInstances[0];

		sender.emit('did-start-navigation', {}, 'app://fluxer/reload', false, true);
		await flushAsyncWork();

		assert.equal(engine.calls.stopCameraPreview, 1);
		assert.equal(engine.calls.clearVideoFrameCallback, 1);
	});

	test('routes camera capture updates through the active session engine', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender},
			{url: 'wss://voice.example.test/update-camera', token: 'update-camera-token'},
		);

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.updateCameraCapture)(
			{sender},
			{deviceId: 'camera-2', backgroundMode: 'blur', backgroundBlurStrength: 80},
		);

		assert.deepEqual(harness.engineInstances[0].calls.updateCameraCapture, [
			[{deviceId: 'camera-2', backgroundMode: 'blur', backgroundBlurStrength: 80}],
		]);
	});

	test('rejects camera capture updates when not connected', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		await assert.rejects(
			() => harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.updateCameraCapture)({sender: makeSender()}, {}),
			/Native voice engine is not connected/,
		);
	});

	test('rejects invalid camera preview args', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)(
					{sender: makeSender()},
					{backgroundMode: 42},
				),
			/Invalid voice-engine start-camera-preview args/,
		);
		assert.equal(harness.engineInstances.length, 0);
	});

	test('drops video frames with a mismatched per-frame bridge version and logs once', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		const sender = makeSender();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender}, {});
		const engine = harness.engineInstances[0];
		const mismatchedMeta = JSON.stringify({
			trackSid: 'local-camera-preview',
			bridgeVersion: VOICE_ENGINE_V2_BRIDGE_VERSION + 1,
		});

		engine.videoFrameCallback(mismatchedMeta, Buffer.from([1, 2, 3]));
		engine.videoFrameCallback(mismatchedMeta, Buffer.from([4, 5, 6]));
		engine.videoFrameCallback(
			JSON.stringify({trackSid: 'local-camera-preview', bridgeVersion: VOICE_ENGINE_V2_BRIDGE_VERSION}),
			Buffer.from([7, 8, 9]),
		);

		assert.equal(sender.sent.length, 1);
		assert.equal(sender.sent[0][0], VOICE_ENGINE_V2_EVENT_CHANNELS.videoFrame);
		assert.equal(sender.sent[0][1].meta.bridgeVersion, VOICE_ENGINE_V2_BRIDGE_VERSION);
		assert.equal(harness.calls.logs.error.length, 1);
		assert.match(harness.calls.logs.error[0][0], /mismatched bridge version/);
	});
});

function tightCameraBackgroundFrame() {
	return {
		format: 'i420',
		width: 4,
		height: 2,
		timestampUs: 1,
		data: new Uint8Array(12).buffer,
	};
}

describe('NativeVoiceEngine camera background frame routing', () => {
	test('routes background frames and clears to the active session engine when connected', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.connect)(
			{sender: makeSender()},
			{url: 'wss://livekit.invalid', token: 'token'},
		);

		const pushed = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushCameraBackgroundFrame)(
			null,
			tightCameraBackgroundFrame(),
		);
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.clearCameraBackgroundFrame)(null);

		const engine = harness.engineInstances[0];
		assert.equal(pushed, true);
		assert.equal(engine.calls.cameraBackgroundFrames.length, 1);
		const frame = engine.calls.cameraBackgroundFrames[0][0];
		assert.equal(frame.format, 'i420');
		assert.equal(frame.width, 4);
		assert.equal(frame.height, 2);
		assert.equal(frame.timestampUs, 1);
		assert.equal(Buffer.isBuffer(frame.data), true);
		assert.equal(frame.data.byteLength, 12);
		assert.equal(engine.calls.clearCameraBackgroundFrame, 1);
	});

	test('routes background frames to the standalone preview engine', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview)({sender: makeSender()}, {});

		const pushed = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushCameraBackgroundFrame)(
			null,
			tightCameraBackgroundFrame(),
		);
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.clearCameraBackgroundFrame)(null);

		const previewEngine = harness.engineInstances[0];
		assert.equal(pushed, true);
		assert.equal(previewEngine.calls.cameraBackgroundFrames.length, 1);
		assert.equal(previewEngine.calls.clearCameraBackgroundFrame, 1);
	});

	test('returns false without a session and rejects invalid frames', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		const pushed = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushCameraBackgroundFrame)(
			null,
			tightCameraBackgroundFrame(),
		);

		assert.equal(pushed, false);
		await assert.rejects(
			() =>
				harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.pushCameraBackgroundFrame)(null, {
					format: 'i420',
					width: 4,
					height: 2,
					timestampUs: 1,
					data: new Uint8Array(11).buffer,
				}),
			/Invalid voice-engine push-camera-background-frame args/,
		);
		assert.equal(harness.engineInstances.length, 0);
	});
});

describe('NativeVoiceEngine readiness', () => {
	test('reports not ready with the load failure detail when the native module fails to load', async () => {
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {loadError: new Error('addon load exploded')},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		const readiness = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();

		assert.deepEqual(plain(readiness), {ready: false, reason: 'addon load exploded'});
		assert.equal(harness.engineInstances.length, 0);
	});

	test('reports not ready when the addon does not expose a VoiceEngine constructor', async () => {
		const harness = loadNativeVoiceEngine({nativeModuleOverrides: {VoiceEngine: undefined}});
		harness.module.registerNativeVoiceEngineHandlers();

		const readiness = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();

		assert.deepEqual(plain(readiness), {ready: false, reason: 'native addon does not expose VoiceEngine'});
	});

	test('reports not-constructed readiness before explicit prewarm', () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();

		const readiness = harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();

		assert.deepEqual(plain(readiness), {ready: false, reason: 'native voice engine not constructed'});
	});

	test('surfaces the prewarm failure detail as the not-ready reason and rejects the prewarm call', async () => {
		let attempts = 0;
		const harness = loadNativeVoiceEngine({
			nativeModuleOverrides: {
				prewarmVoiceEngine: () => {
					attempts += 1;
					throw new Error('prewarm exploded');
				},
			},
		});
		harness.module.registerNativeVoiceEngineHandlers();

		await assert.rejects(harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)(), /prewarm exploded/);

		assert.equal(attempts, 3);
		const readiness = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();
		assert.equal(readiness.ready, false);
		assert.match(readiness.reason, /prewarm exploded/);
	});

	test('reports ready once the singleton is constructed and prewarmed', async () => {
		const harness = loadNativeVoiceEngine();
		harness.module.registerNativeVoiceEngineHandlers();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();

		const readiness = await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness)();

		assert.deepEqual(plain(readiness), {ready: true});
		assert.equal(harness.engineInstances.length, 1);
	});

	test('pushes a one-time engineReady event when readiness first becomes true', async () => {
		const harness = loadNativeVoiceEngine();
		const target = makeSender();
		harness.webContentsList.push(target);
		harness.module.registerNativeVoiceEngineHandlers();

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();
		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();
		await flushAsyncWork();

		const readyEvents = target.sent.filter(([, message]) => message?.type === VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE);
		assert.equal(readyEvents.length, 1);
		assert.equal(readyEvents[0][0], VOICE_ENGINE_V2_EVENT_CHANNELS.event);
		assert.deepEqual(plain(readyEvents[0][1]), {
			type: VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE,
			payload: {ready: true},
		});
	});

	test('does not push engineReady to destroyed web contents', async () => {
		const harness = loadNativeVoiceEngine();
		const destroyed = makeSender();
		destroyed.isDestroyed = () => true;
		const live = makeSender();
		harness.webContentsList.push(destroyed, live);
		harness.module.registerNativeVoiceEngineHandlers();

		await harness.handlers.get(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm)();
		await flushAsyncWork();

		assert.deepEqual(destroyed.sent, []);
		assert.equal(
			live.sent.filter(([, message]) => message?.type === VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE).length,
			1,
		);
	});
});
