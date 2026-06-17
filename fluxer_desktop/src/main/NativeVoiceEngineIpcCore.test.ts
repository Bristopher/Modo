// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import test from 'node:test';
import {
	isVoiceEngineV2BridgePublishMicrophoneOptions,
	type VoiceEngineV2BridgeAudioInputDevice,
	type VoiceEngineV2BridgeAudioOutputDevice,
	type VoiceEngineV2BridgeCameraDevice,
} from '@fluxer/voice_engine_v2/bridge';
import {
	clampNativeMicrophoneMaxBitrateBps,
	handleNativeVoiceEngineListAudioInputDevices,
	handleNativeVoiceEngineListAudioOutputDevices,
	handleNativeVoiceEngineListCameraDevices,
	handleNativeVoiceEnginePublishMicrophone,
	MICROPHONE_MAX_BITRATE_BPS_CAP,
	MICROPHONE_MAX_BITRATE_BPS_FLOOR,
	type NativeMicrophonePublishOptions,
	type NativeVoiceEngineAudioIpcEngine,
	type NativeVoiceEngineAudioQueryEngine,
	NativeVoiceEngineUnavailableError,
} from './NativeVoiceEngineIpcCore';

function createEngine(overrides: Partial<NativeVoiceEngineAudioIpcEngine> = {}): NativeVoiceEngineAudioIpcEngine {
	return {
		publishMicrophone: async () => {},
		...overrides,
	};
}

function createAudioQueryEngine(
	overrides: Partial<NativeVoiceEngineAudioQueryEngine> = {},
): NativeVoiceEngineAudioQueryEngine {
	return {
		listAudioInputDevices: () => [],
		listAudioOutputDevices: () => [],
		...overrides,
	};
}

test('production microphone publish calls publishDeviceMicrophone with device and DSP options', async () => {
	const deviceCalls: Array<NativeMicrophonePublishOptions> = [];
	const pcmCalls: Array<[number, number]> = [];
	const engine = createEngine({
		publishMicrophone: async (sampleRate, channels) => {
			pcmCalls.push([sampleRate, channels]);
		},
		publishDeviceMicrophone: async (options) => {
			deviceCalls.push(options);
		},
	});

	await handleNativeVoiceEnginePublishMicrophone(
		{engine},
		{
			deviceId: 'recording-device-guid',
			echoCancellation: false,
			noiseSuppression: true,
			autoGainControl: false,
		},
	);

	assert.deepEqual(deviceCalls, [
		{
			deviceId: 'recording-device-guid',
			echoCancellation: false,
			noiseSuppression: true,
			autoGainControl: false,
		},
	]);
	assert.deepEqual(pcmCalls, []);
});

test('production microphone publish forwards a clamped maxBitrateBps to the native addon', async () => {
	const deviceCalls: Array<NativeMicrophonePublishOptions> = [];
	const engine = createEngine({
		publishDeviceMicrophone: async (options) => {
			deviceCalls.push(options);
		},
	});

	await handleNativeVoiceEnginePublishMicrophone({engine}, {deviceId: 'recording-device-guid', maxBitrateBps: 96_000});
	await handleNativeVoiceEnginePublishMicrophone({engine}, {deviceId: 'recording-device-guid', maxBitrateBps: 1});
	await handleNativeVoiceEnginePublishMicrophone(
		{engine},
		{deviceId: 'recording-device-guid', maxBitrateBps: 1_000_000},
	);

	assert.deepEqual(
		deviceCalls.map((options) => options.maxBitrateBps),
		[96_000, MICROPHONE_MAX_BITRATE_BPS_FLOOR, MICROPHONE_MAX_BITRATE_BPS_CAP],
	);
});

test('microphone publish omits maxBitrateBps from the native call when absent', async () => {
	const deviceCalls: Array<NativeMicrophonePublishOptions> = [];
	const engine = createEngine({
		publishDeviceMicrophone: async (options) => {
			deviceCalls.push(options);
		},
	});

	await handleNativeVoiceEnginePublishMicrophone({engine}, {deviceId: 'recording-device-guid'});

	assert.equal(deviceCalls.length, 1);
	assert.equal('maxBitrateBps' in (deviceCalls[0] ?? {}), false);
});

test('microphone publish rejects invalid maxBitrateBps values', async () => {
	const engine = createEngine({
		publishDeviceMicrophone: async () => {},
	});

	for (const maxBitrateBps of [0, -64_000, 64_000.5, Number.NaN, Number.POSITIVE_INFINITY]) {
		await assert.rejects(
			() => handleNativeVoiceEnginePublishMicrophone({engine}, {deviceId: 'recording-device-guid', maxBitrateBps}),
			/Invalid voice-engine microphone maxBitrateBps/,
		);
	}
	await assert.rejects(
		() =>
			handleNativeVoiceEnginePublishMicrophone(
				{engine},
				{deviceId: 'recording-device-guid', maxBitrateBps: '96000' as unknown as number},
			),
		/Invalid voice-engine microphone maxBitrateBps/,
	);
});

test('clampNativeMicrophoneMaxBitrateBps pair-asserts the app-side floor and cap', () => {
	assert.equal(clampNativeMicrophoneMaxBitrateBps(undefined), undefined);
	assert.equal(clampNativeMicrophoneMaxBitrateBps(MICROPHONE_MAX_BITRATE_BPS_FLOOR), MICROPHONE_MAX_BITRATE_BPS_FLOOR);
	assert.equal(clampNativeMicrophoneMaxBitrateBps(MICROPHONE_MAX_BITRATE_BPS_CAP), MICROPHONE_MAX_BITRATE_BPS_CAP);
	assert.equal(clampNativeMicrophoneMaxBitrateBps(7_999), MICROPHONE_MAX_BITRATE_BPS_FLOOR);
	assert.equal(clampNativeMicrophoneMaxBitrateBps(510_001), MICROPHONE_MAX_BITRATE_BPS_CAP);
	assert.equal(clampNativeMicrophoneMaxBitrateBps(64_000), 64_000);
});

test('pcm-test microphone publish calls the legacy synthetic PCM method', async () => {
	const deviceCalls: Array<NativeMicrophonePublishOptions> = [];
	const pcmCalls: Array<[number, number]> = [];
	const engine = createEngine({
		publishMicrophone: async (sampleRate, channels) => {
			pcmCalls.push([sampleRate, channels]);
		},
		publishDeviceMicrophone: async (options) => {
			deviceCalls.push(options);
		},
	});

	await handleNativeVoiceEnginePublishMicrophone({engine}, {mode: 'pcm-test', sampleRate: 48_000, numChannels: 2});

	assert.deepEqual(pcmCalls, [[48_000, 2]]);
	assert.deepEqual(deviceCalls, []);
});

test('audio device list handlers parse JSON strings returned by the native addon', async () => {
	const inputDevices: Array<VoiceEngineV2BridgeAudioInputDevice> = [
		{deviceId: 'input-default', label: 'Default microphone', isDefault: true},
		{deviceId: 'input-usb', label: 'USB microphone', isDefault: false},
	];
	const outputDevices: Array<VoiceEngineV2BridgeAudioOutputDevice> = [
		{deviceId: 'output-default', label: 'Default speaker', isDefault: true},
		{deviceId: 'output-headset', label: 'Headset', isDefault: false},
	];
	const engine = createAudioQueryEngine({
		listAudioInputDevices: () => JSON.stringify(inputDevices),
		listAudioOutputDevices: () => JSON.stringify(outputDevices),
	});

	assert.deepEqual(await handleNativeVoiceEngineListAudioInputDevices(engine), inputDevices);
	assert.deepEqual(await handleNativeVoiceEngineListAudioOutputDevices(engine), outputDevices);
});

test('audio device list handlers pass through arrays returned by the native addon', async () => {
	const inputDevices: Array<VoiceEngineV2BridgeAudioInputDevice> = [
		{deviceId: 'input-default', label: 'Default microphone', isDefault: true},
	];
	const outputDevices: Array<VoiceEngineV2BridgeAudioOutputDevice> = [
		{deviceId: 'output-default', label: 'Default speaker', isDefault: true},
	];
	const engine = createAudioQueryEngine({
		listAudioInputDevices: () => inputDevices,
		listAudioOutputDevices: () => outputDevices,
	});

	assert.equal(await handleNativeVoiceEngineListAudioInputDevices(engine), inputDevices);
	assert.equal(await handleNativeVoiceEngineListAudioOutputDevices(engine), outputDevices);
});

test('audio device listing throws when the native voice engine is unavailable', async () => {
	await assert.rejects(() => handleNativeVoiceEngineListAudioInputDevices(null), NativeVoiceEngineUnavailableError);
	await assert.rejects(() => handleNativeVoiceEngineListAudioOutputDevices(null), NativeVoiceEngineUnavailableError);
});

test('audio device listing rejects malformed native addon JSON', async () => {
	const engine = createAudioQueryEngine({
		listAudioInputDevices: () => '{',
		listAudioOutputDevices: () => '{"devices":[]}',
	});

	await assert.rejects(() => handleNativeVoiceEngineListAudioInputDevices(engine), /Failed to parse/);
	await assert.rejects(() => handleNativeVoiceEngineListAudioOutputDevices(engine), /must be a JSON array/);
});

test('microphone publish validation rejects invalid bridge arguments', async () => {
	assert.equal(isVoiceEngineV2BridgePublishMicrophoneOptions(null), false);
	assert.equal(isVoiceEngineV2BridgePublishMicrophoneOptions({mode: 'invalid'}), false);
	assert.equal(
		isVoiceEngineV2BridgePublishMicrophoneOptions({mode: 'pcm-test', sampleRate: '48000', numChannels: 1}),
		false,
	);
	assert.equal(isVoiceEngineV2BridgePublishMicrophoneOptions({deviceId: 7}), false);
	assert.equal(isVoiceEngineV2BridgePublishMicrophoneOptions({mode: 'device', deviceId: 'input-default'}), true);

	await assert.rejects(
		() => handleNativeVoiceEnginePublishMicrophone(null, {mode: 'pcm-test', sampleRate: 48_000, numChannels: 1}),
		/Native voice engine is not connected/,
	);
	await assert.rejects(
		() => handleNativeVoiceEnginePublishMicrophone({engine: createEngine()}, {mode: 'pcm-test', sampleRate: 48_000}),
		/Invalid voice-engine pcm-test microphone args/,
	);
	await assert.rejects(
		() => handleNativeVoiceEnginePublishMicrophone({engine: createEngine()}, {deviceId: 'input-default'}),
		/Native voice engine addon does not support device microphone capture/,
	);
});

test('camera device listing throws when the native voice engine is unavailable', async () => {
	await assert.rejects(() => handleNativeVoiceEngineListCameraDevices(null), NativeVoiceEngineUnavailableError);
});

test('camera device listing returns the native device list without a connected session', async () => {
	const devices: Array<VoiceEngineV2BridgeCameraDevice> = [
		{deviceId: 'native-studio', label: 'Studio Display Camera', description: 'Apple Studio Display Camera', index: 0},
	];

	assert.deepEqual(await handleNativeVoiceEngineListCameraDevices({listCameraDevices: () => devices}), devices);
});
