// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import type {
	VoiceEngineV2BridgeAudioInputDevice,
	VoiceEngineV2BridgeAudioOutputDevice,
	VoiceEngineV2BridgeCameraDevice,
	VoiceEngineV2BridgePublishMicrophoneOptions,
} from '@fluxer/voice_engine_v2/bridge';

export const MICROPHONE_MAX_BITRATE_BPS_FLOOR = 8_000;
export const MICROPHONE_MAX_BITRATE_BPS_CAP = 510_000;

export interface NativeMicrophonePublishOptions {
	deviceId?: string;
	echoCancellation?: boolean;
	noiseSuppression?: boolean;
	autoGainControl?: boolean;
	deepFilter?: boolean;
	deepFilterNoiseReductionLevel?: number;
	maxBitrateBps?: number;
}

interface NativeVoiceEnginePublishMicrophoneArgs extends VoiceEngineV2BridgePublishMicrophoneOptions {
	maxBitrateBps?: number;
}

export interface NativeVoiceEngineAudioIpcEngine {
	publishMicrophone(sampleRate: number, channels: number): Promise<void>;
	publishDeviceMicrophone?(opts: NativeMicrophonePublishOptions): Promise<void>;
}

export interface NativeVoiceEngineAudioQueryEngine {
	listAudioInputDevices():
		| Array<VoiceEngineV2BridgeAudioInputDevice>
		| string
		| Promise<Array<VoiceEngineV2BridgeAudioInputDevice> | string>;
	listAudioOutputDevices():
		| Array<VoiceEngineV2BridgeAudioOutputDevice>
		| string
		| Promise<Array<VoiceEngineV2BridgeAudioOutputDevice> | string>;
}

interface NativeVoiceEngineAudioIpcSession {
	engine: NativeVoiceEngineAudioIpcEngine;
}

export class NativeVoiceEngineCapabilityError extends Error {
	readonly capability: string;

	constructor(capability: string, message: string) {
		super(message);
		this.name = 'NativeVoiceEngineCapabilityError';
		this.capability = capability;
	}
}

export class NativeVoiceEngineNotConnectedError extends Error {
	constructor() {
		super('Native voice engine is not connected');
		this.name = 'NativeVoiceEngineNotConnectedError';
	}
}

export class NativeVoiceEngineUnavailableError extends Error {
	constructor() {
		super('Native voice engine is unavailable');
		this.name = 'NativeVoiceEngineUnavailableError';
	}
}

export class NativeVoiceEngineInvalidArgsError extends Error {
	constructor(message: string) {
		super(message);
		this.name = 'NativeVoiceEngineInvalidArgsError';
	}
}

function parseNativeJsonArray<T>(value: unknown, label: string): Array<T> {
	if (Array.isArray(value)) return value as Array<T>;
	if (typeof value !== 'string') {
		throw new Error(`Native voice engine ${label} must be a JSON array`);
	}
	let parsed: unknown;
	try {
		parsed = JSON.parse(value) as unknown;
	} catch (error) {
		throw new Error(`Failed to parse native voice engine ${label}: ${String(error)}`);
	}
	if (!Array.isArray(parsed)) {
		throw new Error(`Native voice engine ${label} must be a JSON array`);
	}
	return parsed as Array<T>;
}

function clampNativeMicrophoneDeepFilterNoiseReductionLevel(level: number | undefined): number | undefined {
	if (level === undefined) {
		return undefined;
	}
	if (!Number.isFinite(level)) {
		throw new NativeVoiceEngineInvalidArgsError('Invalid voice-engine microphone deepFilterNoiseReductionLevel');
	}
	const clamped = Math.min(Math.max(level, 0), 100);
	assert.ok(clamped >= 0, 'microphone deepFilterNoiseReductionLevel must be at least 0');
	assert.ok(clamped <= 100, 'microphone deepFilterNoiseReductionLevel must be at most 100');
	return clamped;
}

export function clampNativeMicrophoneMaxBitrateBps(maxBitrateBps: number | undefined): number | undefined {
	if (maxBitrateBps === undefined) {
		return undefined;
	}
	if (!Number.isInteger(maxBitrateBps)) {
		throw new NativeVoiceEngineInvalidArgsError('Invalid voice-engine microphone maxBitrateBps');
	}
	if (maxBitrateBps <= 0) {
		throw new NativeVoiceEngineInvalidArgsError('Invalid voice-engine microphone maxBitrateBps');
	}
	const clamped = Math.min(Math.max(maxBitrateBps, MICROPHONE_MAX_BITRATE_BPS_FLOOR), MICROPHONE_MAX_BITRATE_BPS_CAP);
	assert.ok(clamped >= MICROPHONE_MAX_BITRATE_BPS_FLOOR, 'microphone maxBitrateBps must respect the floor');
	assert.ok(clamped <= MICROPHONE_MAX_BITRATE_BPS_CAP, 'microphone maxBitrateBps must respect the cap');
	return clamped;
}

export async function handleNativeVoiceEnginePublishMicrophone(
	session: NativeVoiceEngineAudioIpcSession | null,
	args: NativeVoiceEnginePublishMicrophoneArgs,
): Promise<void> {
	if (!session) {
		throw new NativeVoiceEngineNotConnectedError();
	}
	if (args.mode === 'pcm-test') {
		if (typeof args.sampleRate !== 'number' || typeof args.numChannels !== 'number') {
			throw new NativeVoiceEngineInvalidArgsError('Invalid voice-engine pcm-test microphone args');
		}
		await session.engine.publishMicrophone(args.sampleRate, args.numChannels);
		return;
	}
	if (typeof session.engine.publishDeviceMicrophone !== 'function') {
		throw new NativeVoiceEngineCapabilityError(
			'microphoneCapture',
			'Native voice engine addon does not support device microphone capture',
		);
	}
	const maxBitrateBps = clampNativeMicrophoneMaxBitrateBps(args.maxBitrateBps);
	const deepFilterNoiseReductionLevel = clampNativeMicrophoneDeepFilterNoiseReductionLevel(
		args.deepFilterNoiseReductionLevel,
	);
	await session.engine.publishDeviceMicrophone({
		deviceId: args.deviceId,
		echoCancellation: args.echoCancellation,
		noiseSuppression: args.noiseSuppression,
		autoGainControl: args.autoGainControl,
		...(args.deepFilter !== undefined ? {deepFilter: args.deepFilter} : {}),
		...(deepFilterNoiseReductionLevel !== undefined ? {deepFilterNoiseReductionLevel} : {}),
		...(maxBitrateBps !== undefined ? {maxBitrateBps} : {}),
	});
}

interface NativeVoiceEngineCameraQueryEngine {
	listCameraDevices(): Array<VoiceEngineV2BridgeCameraDevice> | Promise<Array<VoiceEngineV2BridgeCameraDevice>>;
}

export async function handleNativeVoiceEngineListCameraDevices(
	engine: NativeVoiceEngineCameraQueryEngine | null,
): Promise<Array<VoiceEngineV2BridgeCameraDevice>> {
	if (!engine) {
		throw new NativeVoiceEngineUnavailableError();
	}
	const devices = await engine.listCameraDevices();
	assert.ok(Array.isArray(devices), 'native voice engine listCameraDevices must return an array');
	return devices;
}

export async function handleNativeVoiceEngineListAudioOutputDevices(
	engine: NativeVoiceEngineAudioQueryEngine | null,
): Promise<Array<VoiceEngineV2BridgeAudioOutputDevice>> {
	if (!engine) {
		throw new NativeVoiceEngineUnavailableError();
	}
	return parseNativeJsonArray<VoiceEngineV2BridgeAudioOutputDevice>(
		await engine.listAudioOutputDevices(),
		'audio output devices',
	);
}

export async function handleNativeVoiceEngineListAudioInputDevices(
	engine: NativeVoiceEngineAudioQueryEngine | null,
): Promise<Array<VoiceEngineV2BridgeAudioInputDevice>> {
	if (!engine) {
		throw new NativeVoiceEngineUnavailableError();
	}
	return parseNativeJsonArray<VoiceEngineV2BridgeAudioInputDevice>(
		await engine.listAudioInputDevices(),
		'audio input devices',
	);
}
