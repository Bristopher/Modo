// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createChildLogger} from '@electron/common/Logger';
import type {
	VoiceEngineV2BridgeAudioDeviceModuleState,
	VoiceEngineV2BridgeAudioInputDevice,
	VoiceEngineV2BridgeAudioOutputDevice,
	VoiceEngineV2BridgeCameraDevice,
	VoiceEngineV2BridgeCameraPreviewInfo,
	VoiceEngineV2BridgeCapabilities,
	VoiceEngineV2BridgeConnectOptions,
	VoiceEngineV2BridgeFloatPcmFrame,
	VoiceEngineV2BridgeHardwareEncoderCapabilities,
	VoiceEngineV2BridgeOperationErrorCode,
	VoiceEngineV2BridgeOperationResult,
	VoiceEngineV2BridgeParticipantVolumeOptions,
	VoiceEngineV2BridgePcmFrame,
	VoiceEngineV2BridgeProcessedCameraFrame,
	VoiceEngineV2BridgePublishCameraOptions,
	VoiceEngineV2BridgePublishDataOptions,
	VoiceEngineV2BridgePublishDeviceScreenShareOptions,
	VoiceEngineV2BridgePublishMicrophoneOptions,
	VoiceEngineV2BridgePublishNativeCameraSinkResult,
	VoiceEngineV2BridgePublishProcessedCameraOptions,
	VoiceEngineV2BridgePublishProcessedCameraResult,
	VoiceEngineV2BridgePublishScreenAudioOptions,
	VoiceEngineV2BridgePublishScreenOptions,
	VoiceEngineV2BridgeReadiness,
	VoiceEngineV2BridgeRemoteTrackSubscriptionOptions,
	VoiceEngineV2BridgeSpeakingDetectionOptions,
	VoiceEngineV2BridgeStartCameraPreviewOptions,
	VoiceEngineV2BridgeStats,
	VoiceEngineV2BridgeUpdateCameraCaptureOptions,
	VoiceEngineV2BridgeUpdateScreenShareEncodingOptions,
} from '@fluxer/voice_engine_v2/bridge';
import {
	clampVoiceEngineV2ParticipantVolume,
	createVoiceEngineV2OperationFailure,
	isVoiceEngineV2BridgeCameraPreviewInfo,
	isVoiceEngineV2BridgeConnectOptions,
	isVoiceEngineV2BridgeFloatPcmFrame,
	isVoiceEngineV2BridgePcmFrame,
	isVoiceEngineV2BridgeProcessedCameraFrame,
	isVoiceEngineV2BridgePublishCameraOptions,
	isVoiceEngineV2BridgePublishDataOptions,
	isVoiceEngineV2BridgePublishDeviceScreenShareOptions,
	isVoiceEngineV2BridgePublishMicrophoneOptions,
	isVoiceEngineV2BridgePublishProcessedCameraOptions,
	isVoiceEngineV2BridgePublishProcessedCameraResult,
	isVoiceEngineV2BridgePublishScreenAudioOptions,
	isVoiceEngineV2BridgePublishScreenOptions,
	isVoiceEngineV2BridgeRemoteTrackSubscriptionOptions,
	isVoiceEngineV2BridgeSpeakingDetectionOptions,
	isVoiceEngineV2BridgeUpdateScreenShareEncodingOptions,
	isVoiceEngineV2ParticipantVolumeOptions,
	normalizeVoiceEngineV2BridgeCapabilities,
	normalizeVoiceEngineV2BridgeHardwareEncoderCapabilities,
	runVoiceEngineV2AdmWarmup,
	unavailableVoiceEngineV2BridgeCapabilities,
	unavailableVoiceEngineV2BridgeHardwareEncoderCapabilities,
	VOICE_ENGINE_V2_ADM_STATUS_EVENT_TYPE,
	VOICE_ENGINE_V2_BRIDGE_VERSION,
	VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE,
	VOICE_ENGINE_V2_EVENT_CHANNELS,
	VOICE_ENGINE_V2_IPC_CHANNELS,
	VOICE_ENGINE_V2_OPERATION_SUCCESS,
} from '@fluxer/voice_engine_v2/bridge';
import {ipcMain, type WebContents, webContents} from 'electron';
import {
	handleNativeVoiceEngineListAudioInputDevices,
	handleNativeVoiceEngineListAudioOutputDevices,
	handleNativeVoiceEngineListCameraDevices,
	handleNativeVoiceEnginePublishMicrophone,
	type NativeMicrophonePublishOptions,
	NativeVoiceEngineCapabilityError,
	NativeVoiceEngineInvalidArgsError,
	NativeVoiceEngineNotConnectedError,
} from './NativeVoiceEngineIpcCore';
import {acquireStreamingPriority, releaseStreamingPriority} from './StreamingPriority';

const logger = createChildLogger('NativeVoiceEngine');
const requireModule = createRequire(import.meta.url);

interface NativeScreenSharePublishOptions {
	adaptiveSend?: boolean;
	minVideoFps?: number;
	maxAudioBufferMs?: number;
	pacing?: 'sender' | 'source';
	captureId: string;
	trackName?: string;
}

interface NativeVoiceEngineInstance {
	connect(url: string, token: string, e2eeKey?: Buffer): Promise<void>;
	disconnect(): Promise<void>;
	isConnected(): boolean;
	publishScreenShare(
		width: number,
		height: number,
		codec: string,
		maxBitrateBps: number | undefined,
		maxFramerate: number | undefined,
		simulcast: boolean | undefined,
		options: NativeScreenSharePublishOptions,
	): Promise<void>;
	updateScreenShareEncoding?(
		width: number,
		height: number,
		maxBitrateBps: number | undefined,
		maxFramerate: number | undefined,
		options: NativeScreenSharePublishOptions,
	): Promise<void>;
	createScreenFrameSinkHandle?(captureId: string): unknown | null;
	createScreenAudioSinkHandle?(): unknown | null;
	unpublishScreenShare(): Promise<void>;
	isPublishingScreen(): boolean;
	publishScreenShareAudio(sampleRate: number, channels: number): Promise<void>;
	pushScreenSharePcm(buffer: Buffer, sampleRate: number, channels: number): Promise<boolean>;
	pushScreenShareFloat(buffer: Buffer, sampleRate: number, channels: number): Promise<boolean>;
	unpublishScreenShareAudio(): Promise<void>;
	isPublishingScreenAudio(): boolean;
	publishMicrophone(sampleRate: number, channels: number): Promise<void>;
	publishDeviceMicrophone?(opts: NativeMicrophonePublishOptions): Promise<void>;
	pushPcm(buffer: Buffer, sampleRate: number, channels: number): Promise<boolean>;
	setMicEnabled(enabled: boolean): Promise<void>;
	setSpeakingDetection?(localThresholdRms: number, remoteThresholdRms: number): void;
	listAudioInputDevices():
		| Array<VoiceEngineV2BridgeAudioInputDevice>
		| string
		| Promise<Array<VoiceEngineV2BridgeAudioInputDevice> | string>;
	listAudioOutputDevices():
		| Array<VoiceEngineV2BridgeAudioOutputDevice>
		| string
		| Promise<Array<VoiceEngineV2BridgeAudioOutputDevice> | string>;
	setAudioOutputDevice(deviceId: string): Promise<void>;
	ensurePlatformAudio?(): Promise<void>;
	setParticipantVolume(participantSid: string, volume: number): Promise<void>;
	setRemoteTrackSubscription?(options: VoiceEngineV2BridgeRemoteTrackSubscriptionOptions): Promise<void>;
	publishData?(buffer: Buffer, reliable: boolean, topic?: string, destinationIdentities?: Array<string>): Promise<void>;
	listCameraDevices(): Array<VoiceEngineV2BridgeCameraDevice> | Promise<Array<VoiceEngineV2BridgeCameraDevice>>;
	publishCamera(opts: VoiceEngineV2BridgePublishCameraOptions): Promise<void>;
	publishNativeCameraSink?(
		opts: VoiceEngineV2BridgePublishCameraOptions,
	): Promise<VoiceEngineV2BridgePublishNativeCameraSinkResult>;
	publishProcessedCamera(
		opts: VoiceEngineV2BridgePublishProcessedCameraOptions,
	): Promise<VoiceEngineV2BridgePublishProcessedCameraResult>;
	pushProcessedCameraFrame(frame: VoiceEngineV2BridgeProcessedCameraFrame): Promise<boolean>;
	pushCameraBackgroundFrame(frame: VoiceEngineV2BridgeProcessedCameraFrame): Promise<boolean> | boolean;
	clearCameraBackgroundFrame(): void;
	updateCameraCapture?(opts: VoiceEngineV2BridgeUpdateCameraCaptureOptions): Promise<void>;
	publishDeviceScreenShare(opts: VoiceEngineV2BridgePublishDeviceScreenShareOptions): Promise<void>;
	unpublishCamera(): Promise<void>;
	isPublishingCamera(): boolean;
	startCameraPreview?(
		opts: VoiceEngineV2BridgeStartCameraPreviewOptions,
	): Promise<VoiceEngineV2BridgeCameraPreviewInfo>;
	stopCameraPreview?(): void;
	getConnectionStats(): Promise<VoiceEngineV2BridgeStats>;
	droppedVideoFrameCallbacks?(): number;
	setEventCallback(cb: (...args: Array<unknown>) => void): void;
	setVideoFrameCallback?(cb: (...args: Array<unknown>) => void): void;
	clearVideoFrameCallback?(): void;
}

interface WebrtcSenderEngineModule {
	isSupported?: () => boolean;
	getEngineBridgeVersion?: () => number | null;
	assertEngineBridgeVersion?: (version: number) => void;
	getCapabilities?: () => VoiceEngineV2BridgeCapabilities;
	prewarmVoiceEngine?: () => void | Promise<void>;
	probeAudioDeviceModule?: () => Promise<boolean>;
	getHardwareEncoderCapabilities?: () => VoiceEngineV2BridgeHardwareEncoderCapabilities;
	VoiceEngine?: new () => NativeVoiceEngineInstance;
	loadError?: Error | null;
}

const NATIVE_VOICE_ENGINE_PREWARM_ATTEMPTS_MAX = 3;
const NATIVE_VOICE_ENGINE_PREWARM_RETRY_DELAY_MS = 250;

let cachedModule: WebrtcSenderEngineModule | null | undefined;
let cachedModuleLoadErrorDetail: string | undefined;
let nativeVoiceEngineAddonPrewarmed = false;
let nativeVoiceEnginePrewarmFailureDetail: string | undefined;
let nativeVoiceEnginePrewarmPromise: Promise<void> | null = null;
let engineSingleton: NativeVoiceEngineInstance | null = null;
let engineReadyEventSent = false;
let admState: VoiceEngineV2BridgeAudioDeviceModuleState = {status: 'warming'};
let admWarmupRun: Promise<void> | null = null;
let pendingOutputDeviceId: string | null = null;
let pendingOutputDeviceAdmRetryAttempt = 0;
const PENDING_OUTPUT_DEVICE_ADM_RETRY_ATTEMPTS_MAX = 3;

function delayMs(durationMs: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, durationMs));
}

function describeEngineBridgeVersionFailure(stage: string, error: unknown): string {
	const detail = error instanceof Error ? error.message : String(error);
	return `${stage}: ${detail}`;
}

function verifyEngineBridgeVersion(mod: WebrtcSenderEngineModule): string | null {
	if (typeof mod.getEngineBridgeVersion !== 'function') {
		return 'native addon does not export getEngineBridgeVersion';
	}
	if (typeof mod.assertEngineBridgeVersion !== 'function') {
		return 'native addon does not export assertEngineBridgeVersion';
	}
	let addonVersion: number | null;
	try {
		addonVersion = mod.getEngineBridgeVersion();
	} catch (error) {
		return describeEngineBridgeVersionFailure('native addon getEngineBridgeVersion() threw', error);
	}
	if (addonVersion !== VOICE_ENGINE_V2_BRIDGE_VERSION) {
		return `bridge version mismatch: native addon reports ${addonVersion}, host expects ${VOICE_ENGINE_V2_BRIDGE_VERSION}`;
	}
	try {
		mod.assertEngineBridgeVersion(VOICE_ENGINE_V2_BRIDGE_VERSION);
	} catch (error) {
		return describeEngineBridgeVersionFailure(
			`native addon rejected bridge version ${VOICE_ENGINE_V2_BRIDGE_VERSION}`,
			error,
		);
	}
	return null;
}

function loadEngineModule(): WebrtcSenderEngineModule | null {
	if (cachedModule !== undefined) return cachedModule;
	cachedModuleLoadErrorDetail = undefined;
	try {
		const mod = requireModule('@fluxer/webrtc-sender') as WebrtcSenderEngineModule;
		if (mod.loadError) {
			logger.warn('Native webrtc sender addon reported load error', mod.loadError);
			cachedModuleLoadErrorDetail = mod.loadError.message;
			cachedModule = null;
		} else {
			const bridgeVersionFailure = verifyEngineBridgeVersion(mod);
			if (bridgeVersionFailure) {
				logger.error('Native voice engine bridge version check failed; native voice engine disabled', {
					detail: bridgeVersionFailure,
					hostBridgeVersion: VOICE_ENGINE_V2_BRIDGE_VERSION,
				});
				cachedModuleLoadErrorDetail = bridgeVersionFailure;
				cachedModule = null;
			} else if (typeof mod.VoiceEngine !== 'function') {
				logger.info('Native webrtc sender addon does not expose VoiceEngine; native voice engine disabled');
				cachedModule = mod;
			} else {
				cachedModule = mod;
			}
		}
	} catch (error) {
		logger.warn('Failed to load @fluxer/webrtc-sender', error);
		cachedModuleLoadErrorDetail = error instanceof Error ? error.message : String(error);
		cachedModule = null;
	}
	return cachedModule;
}

function getVoiceEngineV2BridgeCapabilities(): VoiceEngineV2BridgeCapabilities {
	const mod = loadEngineModule();
	if (!mod) return unavailableVoiceEngineV2BridgeCapabilities();
	if (typeof mod.getCapabilities !== 'function') return unavailableVoiceEngineV2BridgeCapabilities();
	try {
		return normalizeVoiceEngineV2BridgeCapabilities(mod.getCapabilities());
	} catch (error) {
		logger.warn('Native webrtc sender getCapabilities() threw', error);
		return unavailableVoiceEngineV2BridgeCapabilities();
	}
}

function ensureEngine(): NativeVoiceEngineInstance | null {
	if (engineSingleton) return engineSingleton;
	const mod = loadEngineModule();
	if (!mod || typeof mod.VoiceEngine !== 'function') return null;
	engineSingleton = new mod.VoiceEngine();
	logger.info('Native voice engine singleton constructed');
	return engineSingleton;
}

async function runNativeVoiceEngineAddonPrewarm(mod: WebrtcSenderEngineModule): Promise<void> {
	if (typeof mod.prewarmVoiceEngine !== 'function') {
		nativeVoiceEngineAddonPrewarmed = true;
		return;
	}
	for (let attempt = 1; attempt <= NATIVE_VOICE_ENGINE_PREWARM_ATTEMPTS_MAX; attempt++) {
		try {
			await mod.prewarmVoiceEngine();
			nativeVoiceEngineAddonPrewarmed = true;
			nativeVoiceEnginePrewarmFailureDetail = undefined;
			logger.info('Native voice engine addon prewarmed', {attempt});
			return;
		} catch (error) {
			nativeVoiceEnginePrewarmFailureDetail = error instanceof Error ? error.message : String(error);
			logger.warn('Native voice engine addon prewarm attempt failed', {
				attempt,
				maxAttempts: NATIVE_VOICE_ENGINE_PREWARM_ATTEMPTS_MAX,
				error,
			});
			if (attempt < NATIVE_VOICE_ENGINE_PREWARM_ATTEMPTS_MAX) {
				await delayMs(NATIVE_VOICE_ENGINE_PREWARM_RETRY_DELAY_MS);
			}
		}
	}
	throw new Error(
		`Native voice engine prewarm failed after ${NATIVE_VOICE_ENGINE_PREWARM_ATTEMPTS_MAX} attempts: ${nativeVoiceEnginePrewarmFailureDetail}`,
	);
}

async function runNativeVoiceEnginePrewarm(): Promise<void> {
	const engine = ensureEngine();
	if (!engine) return;
	const mod = loadEngineModule();
	if (!mod) return;
	if (!nativeVoiceEngineAddonPrewarmed) {
		await runNativeVoiceEngineAddonPrewarm(mod);
	}
	emitEngineReadyEventOnce();
	startAdmWarmup();
}

export function prewarmNativeVoiceEngine(): Promise<void> {
	if (!nativeVoiceEnginePrewarmPromise) {
		nativeVoiceEnginePrewarmPromise = runNativeVoiceEnginePrewarm().finally(() => {
			nativeVoiceEnginePrewarmPromise = null;
		});
	}
	return nativeVoiceEnginePrewarmPromise;
}

async function ensureEngineReady(): Promise<NativeVoiceEngineInstance> {
	const engine = ensureEngine();
	if (!engine) {
		throw new Error('Native voice engine unavailable');
	}
	if (!nativeVoiceEngineAddonPrewarmed) {
		await prewarmNativeVoiceEngine();
	}
	return engine;
}

function isTransientAudioDeviceModuleError(error: unknown): boolean {
	const message = error instanceof Error ? error.message : String(error);
	return message.includes('negative audio device count');
}

export function getAudioDeviceModuleState(): VoiceEngineV2BridgeAudioDeviceModuleState {
	return admState;
}

function broadcastAdmStatusEvent(state: VoiceEngineV2BridgeAudioDeviceModuleState): void {
	const targets = typeof webContents?.getAllWebContents === 'function' ? webContents.getAllWebContents() : [];
	for (const contents of targets) {
		if (contents.isDestroyed()) continue;
		try {
			contents.send(VOICE_ENGINE_V2_EVENT_CHANNELS.event, {
				type: VOICE_ENGINE_V2_ADM_STATUS_EVENT_TYPE,
				payload: state,
			});
		} catch (error) {
			logger.warn('Failed to broadcast audio device module status event', error);
		}
	}
}

function applyAdmTransition(state: VoiceEngineV2BridgeAudioDeviceModuleState): void {
	admState = state;
	broadcastAdmStatusEvent(state);
	if (state.status === 'failed') {
		logger.error('Native voice engine audio device module warmup failed', {detail: state.detail});
		return;
	}
	if (state.status !== 'ready') return;
	logger.info('Native voice engine audio device module ready');
	void applyPendingOutputDevice();
	void healActiveSessionPlatformAudio();
}

function startAdmWarmup(): void {
	if (admWarmupRun) return;
	if (admState.status === 'ready') return;
	const mod = loadEngineModule();
	if (!mod) {
		applyAdmTransition({
			status: 'failed',
			detail: cachedModuleLoadErrorDetail ?? 'native voice engine module unavailable',
		});
		return;
	}
	const probe = mod.probeAudioDeviceModule;
	if (typeof probe !== 'function') {
		applyAdmTransition({status: 'ready'});
		return;
	}
	if (admState.status !== 'warming') {
		admState = {status: 'warming'};
		broadcastAdmStatusEvent(admState);
	}
	admWarmupRun = runVoiceEngineV2AdmWarmup({probe: () => probe(), delay: delayMs})
		.then(applyAdmTransition)
		.finally(() => {
			admWarmupRun = null;
		});
}

function revertAdmToWarming(error: unknown): void {
	logger.warn('Native voice engine audio device module reported not-ready; reverting to warming', {error});
	admState = {status: 'warming'};
	broadcastAdmStatusEvent(admState);
	if (admWarmupRun) {
		void admWarmupRun.finally(() => {
			if (admState.status === 'warming') startAdmWarmup();
		});
		return;
	}
	startAdmWarmup();
}

async function applyPendingOutputDevice(): Promise<void> {
	const deviceId = pendingOutputDeviceId;
	const retryAttempt = pendingOutputDeviceAdmRetryAttempt;
	pendingOutputDeviceId = null;
	pendingOutputDeviceAdmRetryAttempt = 0;
	if (!deviceId) return;
	try {
		const engine = await ensureEngineReady();
		await engine.setAudioOutputDevice(deviceId);
		logger.info('Applied pending audio output device after warmup', {deviceId});
	} catch (error) {
		if (isTransientAudioDeviceModuleError(error)) {
			const nextRetryAttempt = retryAttempt + 1;
			if (nextRetryAttempt <= PENDING_OUTPUT_DEVICE_ADM_RETRY_ATTEMPTS_MAX) {
				pendingOutputDeviceId = deviceId;
				pendingOutputDeviceAdmRetryAttempt = nextRetryAttempt;
				revertAdmToWarming(error);
				return;
			}
		}
		logger.warn('Failed to apply pending audio output device after warmup', {deviceId, error});
	}
}

async function healActiveSessionPlatformAudio(): Promise<void> {
	const session = activeSession;
	if (!session) return;
	if (typeof session.engine.ensurePlatformAudio !== 'function') return;
	try {
		await session.engine.ensurePlatformAudio();
	} catch (error) {
		logger.warn('Failed to bind platform audio after warmup', error);
	}
}

function getVoiceEngineReadiness(): VoiceEngineV2BridgeReadiness {
	const mod = loadEngineModule();
	if (!mod) {
		return {ready: false, reason: cachedModuleLoadErrorDetail ?? 'native voice engine module unavailable'};
	}
	if (typeof mod.VoiceEngine !== 'function') {
		return {ready: false, reason: 'native addon does not expose VoiceEngine'};
	}
	if (!engineSingleton) {
		return {ready: false, reason: 'native voice engine not constructed'};
	}
	if (!nativeVoiceEngineAddonPrewarmed) {
		return {
			ready: false,
			reason: nativeVoiceEnginePrewarmFailureDetail ?? 'native voice engine prewarm pending',
		};
	}
	return {ready: true};
}

function emitEngineReadyEventOnce(): void {
	if (engineReadyEventSent) return;
	const readiness = getVoiceEngineReadiness();
	if (!readiness.ready) return;
	engineReadyEventSent = true;
	const targets = typeof webContents?.getAllWebContents === 'function' ? webContents.getAllWebContents() : [];
	for (const contents of targets) {
		if (contents.isDestroyed()) continue;
		try {
			contents.send(VOICE_ENGINE_V2_EVENT_CHANNELS.event, {
				type: VOICE_ENGINE_V2_ENGINE_READY_EVENT_TYPE,
				payload: {ready: true},
			});
		} catch (error) {
			logger.warn('Failed to broadcast native voice engine engineReady event', error);
		}
	}
	logger.info('Native voice engine ready');
}

function getHardwareEncoderCapabilities(): VoiceEngineV2BridgeHardwareEncoderCapabilities {
	const mod = loadEngineModule();
	if (!mod) {
		return unavailableVoiceEngineV2BridgeHardwareEncoderCapabilities('load-failed', cachedModuleLoadErrorDetail);
	}
	if (typeof mod.getHardwareEncoderCapabilities !== 'function') {
		return unavailableVoiceEngineV2BridgeHardwareEncoderCapabilities('unsupported-addon-version');
	}
	try {
		return normalizeVoiceEngineV2BridgeHardwareEncoderCapabilities(mod.getHardwareEncoderCapabilities());
	} catch (error) {
		logger.warn('Native webrtc sender getHardwareEncoderCapabilities() threw', error);
		const detail = error instanceof Error ? error.message : String(error);
		return unavailableVoiceEngineV2BridgeHardwareEncoderCapabilities('query-failed', detail);
	}
}

function isNativeVoiceEngineSupported(): boolean {
	const mod = loadEngineModule();
	if (!mod || typeof mod.VoiceEngine !== 'function') return false;
	if (typeof mod.isSupported === 'function') {
		try {
			return mod.isSupported();
		} catch (error) {
			logger.warn('Native webrtc sender isSupported() threw', error);
			return false;
		}
	}
	return true;
}

interface ActiveEngineSession {
	engine: NativeVoiceEngineInstance;
	sender: WebContents;
	screenCaptureId: string | null;
	screenPublishOptions: VoiceEngineV2BridgePublishScreenOptions | null;
	streamingPriorityHeld: boolean;
	senderLifecycle: WebContentsOwnerLifecycle | null;
}

let activeSession: ActiveEngineSession | null = null;

export function hasActiveNativeEngineForSender(senderId: number): boolean {
	const session = activeSession;
	return session !== null && session.sender.id === senderId;
}

export function createScreenAudioSinkHandleForSender(senderId: number): unknown {
	const session = activeSession;
	if (!session || session.sender.id !== senderId) return null;
	if (typeof session.engine.createScreenAudioSinkHandle !== 'function') return null;
	try {
		return session.engine.createScreenAudioSinkHandle() ?? null;
	} catch {
		return null;
	}
}

function acquireSessionStreamingPriority(session: ActiveEngineSession): void {
	if (session.streamingPriorityHeld) return;
	acquireStreamingPriority(session.sender);
	session.streamingPriorityHeld = true;
}

function releaseSessionStreamingPriority(session: ActiveEngineSession): void {
	if (!session.streamingPriorityHeld) return;
	releaseStreamingPriority();
	session.streamingPriorityHeld = false;
}

function unpackNapiPair<A, B>(args: ReadonlyArray<unknown>): [A | undefined, B | undefined] {
	if (args.length === 1 && Array.isArray(args[0])) {
		const tuple = args[0] as ReadonlyArray<unknown>;
		return [tuple[0] as A, tuple[1] as B];
	}
	return [args[0] as A, args[1] as B];
}

function forwardEngineEvent(sender: WebContents, type: string, json: string): void {
	if (sender.isDestroyed()) return;
	let payload: unknown = {};
	try {
		payload = json ? JSON.parse(json) : {};
	} catch (error) {
		logger.warn('Failed to parse native voice engine event payload', {type, error});
		payload = {};
	}
	try {
		sender.send(VOICE_ENGINE_V2_EVENT_CHANNELS.event, {type, payload});
	} catch (error) {
		logger.warn('Failed to forward native voice engine event to renderer', {type, error});
	}
}

interface VideoFrameVersionLatch {
	mismatchLogged: boolean;
}

function forwardVideoFrame(
	sender: WebContents,
	metaJson: string,
	buffer: Buffer,
	versionLatch: VideoFrameVersionLatch,
): void {
	if (sender.isDestroyed()) return;
	let meta: unknown = {};
	try {
		meta = metaJson ? JSON.parse(metaJson) : {};
	} catch (error) {
		logger.warn('Failed to parse native voice engine video-frame meta', {error});
		return;
	}
	const frameBridgeVersion = (meta as {bridgeVersion?: unknown}).bridgeVersion;
	if (frameBridgeVersion !== undefined && frameBridgeVersion !== VOICE_ENGINE_V2_BRIDGE_VERSION) {
		if (!versionLatch.mismatchLogged) {
			versionLatch.mismatchLogged = true;
			logger.error('Dropping native voice engine video frames with mismatched bridge version', {
				frameBridgeVersion,
				hostBridgeVersion: VOICE_ENGINE_V2_BRIDGE_VERSION,
			});
		}
		return;
	}
	try {
		sender.send(VOICE_ENGINE_V2_EVENT_CHANNELS.videoFrame, {meta, data: buffer});
	} catch (error) {
		logger.warn('Failed to forward native voice engine video frame to renderer', {error});
	}
}

interface PreviewSessionRecord {
	sender: WebContents;
	senderLifecycle: WebContentsOwnerLifecycle | null;
}

let previewSession: PreviewSessionRecord | null = null;

type WebContentsInvalidationReason = 'destroyed' | 'render-process-gone' | 'main-frame-navigation';

interface WebContentsOwnerLifecycle {
	onDestroyed: () => void;
	onRenderProcessGone: () => void;
	onDidStartNavigation: (event: unknown, url: string, isInPlace: boolean, isMainFrame: boolean) => void;
}

function attachWebContentsOwnerLifecycle(
	sender: WebContents,
	onInvalidated: (reason: WebContentsInvalidationReason) => void,
): WebContentsOwnerLifecycle {
	const lifecycle: WebContentsOwnerLifecycle = {
		onDestroyed: () => onInvalidated('destroyed'),
		onRenderProcessGone: () => onInvalidated('render-process-gone'),
		onDidStartNavigation: (_event, _url, isInPlace, isMainFrame) => {
			if (!isMainFrame) return;
			if (isInPlace) return;
			onInvalidated('main-frame-navigation');
		},
	};
	sender.once('destroyed', lifecycle.onDestroyed);
	sender.once('render-process-gone', lifecycle.onRenderProcessGone);
	sender.on('did-start-navigation', lifecycle.onDidStartNavigation);
	return lifecycle;
}

function detachWebContentsOwnerLifecycle(sender: WebContents, lifecycle: WebContentsOwnerLifecycle | null): void {
	if (!lifecycle) return;
	sender.removeListener('destroyed', lifecycle.onDestroyed);
	sender.removeListener('render-process-gone', lifecycle.onRenderProcessGone);
	sender.removeListener('did-start-navigation', lifecycle.onDidStartNavigation);
}

function registerPreviewVideoFrameForwarding(engine: NativeVoiceEngineInstance, session: PreviewSessionRecord): void {
	if (typeof engine.setVideoFrameCallback !== 'function') {
		logger.info('Native voice engine addon does not expose setVideoFrameCallback; camera preview frames disabled');
		return;
	}
	const versionLatch: VideoFrameVersionLatch = {mismatchLogged: false};
	try {
		engine.setVideoFrameCallback((...args) => {
			if (previewSession !== session) return;
			const [metaJson, buffer] = unpackNapiPair<string, Buffer>(args);
			if (typeof metaJson !== 'string' || !Buffer.isBuffer(buffer)) return;
			forwardVideoFrame(session.sender, metaJson, buffer, versionLatch);
		});
	} catch (error) {
		logger.warn('Failed to register native voice engine preview video-frame callback', error);
	}
}

function stopPreviewSession(): void {
	const session = previewSession;
	if (!session) return;
	previewSession = null;
	detachWebContentsOwnerLifecycle(session.sender, session.senderLifecycle);
	const engine = engineSingleton;
	if (!engine) return;
	try {
		engine.stopCameraPreview?.();
	} catch (error) {
		logger.warn('Native voice engine stopCameraPreview during preview teardown failed', error);
	}
	if (!activeSession) {
		detachEngineCallbacks(engine);
	}
}

function registerVideoFrameForwarding(engine: NativeVoiceEngineInstance, session: ActiveEngineSession): void {
	if (typeof engine.setVideoFrameCallback !== 'function') {
		logger.info('Native voice engine addon does not expose setVideoFrameCallback; inbound video tiles disabled');
		return;
	}
	const versionLatch: VideoFrameVersionLatch = {mismatchLogged: false};
	try {
		engine.setVideoFrameCallback((...args) => {
			if (activeSession !== session) return;
			const [metaJson, buffer] = unpackNapiPair<string, Buffer>(args);
			if (typeof metaJson !== 'string' || !Buffer.isBuffer(buffer)) return;
			forwardVideoFrame(session.sender, metaJson, buffer, versionLatch);
		});
	} catch (error) {
		logger.warn('Failed to register native voice engine video-frame callback', error);
	}
}

function detachEngineCallbacks(engine: NativeVoiceEngineInstance): void {
	try {
		engine.setEventCallback(() => {});
	} catch (error) {
		logger.warn('Failed to detach native voice engine event callback', error);
	}
	if (typeof engine.clearVideoFrameCallback === 'function') {
		try {
			engine.clearVideoFrameCallback();
			return;
		} catch (error) {
			logger.warn('Failed to clear native voice engine video-frame callback', error);
		}
	}
	if (typeof engine.setVideoFrameCallback === 'function') {
		try {
			engine.setVideoFrameCallback(() => {});
		} catch (error) {
			logger.warn('Failed to detach native voice engine video-frame callback', error);
		}
	}
}

function detachSessionCallbacks(session: ActiveEngineSession): void {
	detachEngineCallbacks(session.engine);
}

async function teardownActiveSession(): Promise<void> {
	const session = activeSession;
	if (!session) return;
	activeSession = null;
	releaseSessionStreamingPriority(session);
	detachSessionCallbacks(session);
	detachWebContentsOwnerLifecycle(session.sender, session.senderLifecycle);
	try {
		await session.engine.disconnect();
	} catch (error) {
		logger.warn('Native voice engine disconnect during teardown failed', error);
	}
}

let connectGeneration = 0;
let connectAttemptChain: Promise<void> = Promise.resolve();
let unsettledConnectAttempts = 0;

async function handleConnect(sender: WebContents, args: VoiceEngineV2BridgeConnectOptions): Promise<void> {
	const engine = await ensureEngineReady();
	connectGeneration += 1;
	const generation = connectGeneration;
	const replacingPendingConnect = unsettledConnectAttempts > 0;
	if (replacingPendingConnect) {
		void teardownActiveSession();
	}
	const previousAttemptSettled = replacingPendingConnect ? Promise.resolve() : connectAttemptChain;
	unsettledConnectAttempts += 1;
	const attempt = runConnectAttempt(engine, sender, args, generation, previousAttemptSettled).finally(() => {
		unsettledConnectAttempts -= 1;
	});
	connectAttemptChain = attempt.then(
		() => undefined,
		() => undefined,
	);
	await attempt;
}

async function runConnectAttempt(
	engine: NativeVoiceEngineInstance,
	sender: WebContents,
	args: VoiceEngineV2BridgeConnectOptions,
	generation: number,
	previousAttemptSettled: Promise<void>,
): Promise<void> {
	await previousAttemptSettled;
	if (generation !== connectGeneration) {
		throw new Error('Native voice engine connect superseded');
	}
	await teardownActiveSession();
	stopPreviewSession();
	if (generation !== connectGeneration) {
		throw new Error('Native voice engine connect superseded');
	}
	await dialConnectSession(engine, sender, args, generation);
}

async function dialConnectSession(
	engine: NativeVoiceEngineInstance,
	sender: WebContents,
	args: VoiceEngineV2BridgeConnectOptions,
	generation: number,
): Promise<void> {
	const session: ActiveEngineSession = {
		engine,
		sender,
		screenCaptureId: null,
		screenPublishOptions: null,
		streamingPriorityHeld: false,
		senderLifecycle: null,
	};
	session.senderLifecycle = attachWebContentsOwnerLifecycle(sender, (reason) => {
		if (activeSession !== session) return;
		logger.info('Native voice engine owner renderer invalidated; tearing down session', {reason});
		void teardownActiveSession();
	});
	engine.setEventCallback((...callbackArgs) => {
		if (activeSession !== session) return;
		const [type, json] = unpackNapiPair<string, string>(callbackArgs);
		if (typeof type !== 'string') return;
		forwardEngineEvent(sender, type, typeof json === 'string' ? json : '{}');
	});
	activeSession = session;
	registerVideoFrameForwarding(engine, session);
	let e2eeKey: Buffer | undefined;
	if (args.e2eeKey instanceof ArrayBuffer) {
		e2eeKey = Buffer.from(args.e2eeKey);
	}
	try {
		await engine.connect(args.url, args.token, e2eeKey);
	} catch (error) {
		if (activeSession === session) {
			activeSession = null;
			detachWebContentsOwnerLifecycle(sender, session.senderLifecycle);
			detachSessionCallbacks(session);
		}
		throw error;
	}
	await adoptConnectedSession(engine, session, generation);
}

async function adoptConnectedSession(
	engine: NativeVoiceEngineInstance,
	session: ActiveEngineSession,
	generation: number,
): Promise<void> {
	const superseded = activeSession !== session || session.sender.isDestroyed() || generation !== connectGeneration;
	if (!superseded) {
		logger.info('Native voice engine connected');
		return;
	}
	const ownsActiveSession = activeSession === session;
	if (ownsActiveSession) {
		activeSession = null;
		detachWebContentsOwnerLifecycle(session.sender, session.senderLifecycle);
		detachSessionCallbacks(session);
		try {
			await engine.disconnect();
		} catch (error) {
			logger.warn('Stale native voice engine disconnect after connect completion failed', error);
		}
	}
	detachWebContentsOwnerLifecycle(session.sender, session.senderLifecycle);
	throw new Error('Native voice engine connect completed after its session was replaced');
}

function assertNativeScreenShareZeroCopyRequest(
	args: Pick<VoiceEngineV2BridgePublishScreenOptions, 'codec' | 'hardwareEncoding' | 'zeroCopyRequired'>,
): void {
	if (args.zeroCopyRequired === false) {
		throw new Error('Native voice engine screen-share requires zero-copy transport');
	}
	if (args.hardwareEncoding === true && args.zeroCopyRequired !== true) {
		throw new Error('Native voice engine hardware screen encoding requires zero-copy transport');
	}
	if (args.hardwareEncoding === true) {
		const capabilities = getHardwareEncoderCapabilities();
		if (!capabilities.available || !capabilities.zeroCopy || capabilities.nativeInputs.length === 0) {
			throw new Error('Native voice engine hardware screen encoding requires zero-copy-capable native input');
		}
	}
}

async function handlePublishScreen(args: VoiceEngineV2BridgePublishScreenOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	const screenOptions: VoiceEngineV2BridgePublishScreenOptions = {
		...args,
		zeroCopyRequired: args.zeroCopyRequired ?? true,
	};
	assertNativeScreenShareZeroCopyRequest(screenOptions);
	acquireSessionStreamingPriority(session);
	try {
		await session.engine.publishScreenShare(
			screenOptions.width,
			screenOptions.height,
			screenOptions.codec ?? '',
			screenOptions.maxBitrateBps,
			screenOptions.maxFramerate,
			undefined,
			{
				adaptiveSend: screenOptions.adaptiveSend,
				minVideoFps: screenOptions.minVideoFps,
				maxAudioBufferMs: screenOptions.maxAudioBufferMs,
				pacing: screenOptions.pacing,
				captureId: screenOptions.captureId,
				trackName: screenOptions.trackName,
			},
		);
	} catch (error) {
		releaseSessionStreamingPriority(session);
		throw error;
	}
	session.screenCaptureId = screenOptions.captureId;
	session.screenPublishOptions = screenOptions;
	logger.info('Native voice engine screen-share published', {captureId: screenOptions.captureId});
}

async function handleUpdateScreenShareEncoding(
	args: VoiceEngineV2BridgeUpdateScreenShareEncodingOptions,
): Promise<void> {
	const session = activeSession;
	if (!session?.screenCaptureId || !session.screenPublishOptions) {
		throw new Error('Native voice engine is not publishing screen share');
	}
	if (args.captureId !== session.screenCaptureId) {
		throw new Error('Native voice engine screen-share capture id mismatch');
	}
	const previous = session.screenPublishOptions;
	const next: VoiceEngineV2BridgePublishScreenOptions = {
		...previous,
		width: args.width,
		height: args.height,
		codec: args.codec ?? previous.codec,
		hardwareEncoding: args.hardwareEncoding ?? previous.hardwareEncoding,
		zeroCopyRequired: args.zeroCopyRequired ?? previous.zeroCopyRequired ?? true,
		maxBitrateBps: args.maxBitrateBps ?? previous.maxBitrateBps,
		maxFramerate: args.frameRate ?? previous.maxFramerate,
	};
	assertNativeScreenShareZeroCopyRequest(next);
	const publishOptions: NativeScreenSharePublishOptions = {
		...(next.adaptiveSend !== undefined ? {adaptiveSend: next.adaptiveSend} : {}),
		...(next.minVideoFps !== undefined ? {minVideoFps: next.minVideoFps} : {}),
		...(next.maxAudioBufferMs !== undefined ? {maxAudioBufferMs: next.maxAudioBufferMs} : {}),
		...(next.pacing !== undefined ? {pacing: next.pacing} : {}),
		captureId: next.captureId,
	};
	const codecChanged = args.codec !== undefined && args.codec !== previous.codec;
	const hardwareEncodingChanged =
		args.hardwareEncoding !== undefined && args.hardwareEncoding !== previous.hardwareEncoding;
	if (codecChanged || hardwareEncodingChanged) {
		throw new Error('Native screen-share encoding update cannot change publication codec or hardware mode');
	}
	if (typeof session.engine.updateScreenShareEncoding !== 'function') {
		throw new Error('Native screen-share encoding update is unavailable');
	}
	await session.engine.updateScreenShareEncoding(
		next.width,
		next.height,
		next.maxBitrateBps,
		next.maxFramerate,
		publishOptions,
	);
	session.screenPublishOptions = next;
	logger.info('Native voice engine screen-share encoding updated', {
		captureId: next.captureId,
		width: next.width,
		height: next.height,
		maxBitrateBps: next.maxBitrateBps,
		maxFramerate: next.maxFramerate,
	});
}

async function handleUnpublishScreen(): Promise<void> {
	const session = activeSession;
	if (!session) return;
	if (session.screenCaptureId) {
		session.screenCaptureId = null;
	}
	session.screenPublishOptions = null;
	try {
		await session.engine.unpublishScreenShare();
	} catch (error) {
		logger.warn('Native voice engine unpublishScreenShare failed', error);
	} finally {
		releaseSessionStreamingPriority(session);
	}
}

export function createNativeVoiceEngineScreenFrameSinkHandle(captureId: string): unknown | null {
	const session = activeSession;
	if (!session || session.screenCaptureId !== captureId) return null;
	if (typeof session.engine.createScreenFrameSinkHandle !== 'function') return null;
	try {
		return session.engine.createScreenFrameSinkHandle(captureId) ?? null;
	} catch (error) {
		logger.warn('Failed to create native voice engine screen frame sink handle', {captureId, error});
		return null;
	}
}

function voiceEngineOperationFailure(
	code: VoiceEngineV2BridgeOperationErrorCode,
	message: string,
	capability?: string,
): VoiceEngineV2BridgeOperationResult {
	return createVoiceEngineV2OperationFailure({
		code,
		message,
		...(capability ? {capability} : {}),
	});
}

function mapVoiceEngineOperationError(error: unknown, fallbackCapability?: string): VoiceEngineV2BridgeOperationResult {
	if (error instanceof NativeVoiceEngineCapabilityError) {
		return voiceEngineOperationFailure('unsupported-capability', error.message, error.capability);
	}
	if (error instanceof NativeVoiceEngineNotConnectedError) {
		return voiceEngineOperationFailure('not-connected', error.message);
	}
	if (error instanceof NativeVoiceEngineInvalidArgsError) {
		return voiceEngineOperationFailure('invalid-args', error.message, fallbackCapability);
	}
	const message = error instanceof Error ? error.message : String(error);
	const normalizedMessage = message.replace(/^Error:\s*/, '');
	if (normalizedMessage === 'not connected' || normalizedMessage === 'Native voice engine is not connected') {
		return voiceEngineOperationFailure('not-connected', normalizedMessage, fallbackCapability);
	}
	return voiceEngineOperationFailure('native-error', message, fallbackCapability);
}

function audioDeviceModuleNotReadyFailure(capability: string): VoiceEngineV2BridgeOperationResult | null {
	assert.ok(capability.length > 0, 'ADM preflight capability must not be empty');
	if (admState.status === 'ready') return null;
	if (admState.status === 'warming') {
		startAdmWarmup();
		return voiceEngineOperationFailure('native-error', 'Native audio device module is warming', capability);
	}
	return voiceEngineOperationFailure(
		'native-error',
		admState.detail ?? 'Native audio device module failed to become ready',
		capability,
	);
}

async function runVoiceEngineOperation(
	action: () => Promise<void>,
	fallbackCapability?: string,
): Promise<VoiceEngineV2BridgeOperationResult> {
	try {
		await action();
		return VOICE_ENGINE_V2_OPERATION_SUCCESS;
	} catch (error) {
		return mapVoiceEngineOperationError(error, fallbackCapability);
	}
}

async function handleSetMicEnabled(enabled: boolean): Promise<VoiceEngineV2BridgeOperationResult> {
	const session = activeSession;
	if (!session) {
		return enabled
			? voiceEngineOperationFailure('not-connected', 'Native voice engine is not connected', 'microphoneCapture')
			: VOICE_ENGINE_V2_OPERATION_SUCCESS;
	}
	if (enabled) {
		const admFailure = audioDeviceModuleNotReadyFailure('microphoneCapture');
		if (admFailure) return admFailure;
	}
	return runVoiceEngineOperation(() => session.engine.setMicEnabled(enabled), 'microphoneCapture');
}

async function handlePublishMicrophone(
	args: VoiceEngineV2BridgePublishMicrophoneOptions,
): Promise<VoiceEngineV2BridgeOperationResult> {
	const capability = args.mode === 'pcm-test' ? 'syntheticMicrophonePcm' : 'microphoneCapture';
	const session = activeSession;
	if (!session) return voiceEngineOperationFailure('not-connected', 'Native voice engine is not connected', capability);
	if (args.mode !== 'pcm-test') {
		const admFailure = audioDeviceModuleNotReadyFailure(capability);
		if (admFailure) return admFailure;
	}
	return runVoiceEngineOperation(() => handleNativeVoiceEnginePublishMicrophone(session, args), capability);
}

async function handleSetSpeakingDetection(options: VoiceEngineV2BridgeSpeakingDetectionOptions): Promise<void> {
	const engine = await ensureEngineReady();
	if (typeof engine.setSpeakingDetection !== 'function') {
		throw new Error('Native voice engine addon does not support speaking detection');
	}
	engine.setSpeakingDetection(options.localThresholdRms, options.remoteThresholdRms);
}

function assertPcmFrameArgs(args: unknown, label: string): VoiceEngineV2BridgePcmFrame {
	if (!isVoiceEngineV2BridgePcmFrame(args)) {
		throw new Error(`Invalid voice-engine ${label} args`);
	}
	return args;
}

function assertFloatPcmFrameArgs(args: unknown, label: string): VoiceEngineV2BridgeFloatPcmFrame {
	if (!isVoiceEngineV2BridgeFloatPcmFrame(args)) {
		throw new Error(`Invalid voice-engine ${label} args`);
	}
	return args;
}

async function handlePushPcm(args: unknown): Promise<boolean> {
	const session = activeSession;
	if (!session) return false;
	const frame = assertPcmFrameArgs(args, 'push-pcm');
	return session.engine.pushPcm(Buffer.from(frame.samples), frame.sampleRate, frame.numChannels);
}

async function handlePublishScreenShareAudio(args: VoiceEngineV2BridgePublishScreenAudioOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	await session.engine.publishScreenShareAudio(args.sampleRate, args.numChannels);
}

async function handlePushScreenSharePcm(args: unknown): Promise<boolean> {
	const session = activeSession;
	if (!session) return false;
	const frame = assertPcmFrameArgs(args, 'push-screen-audio-pcm');
	return session.engine.pushScreenSharePcm(Buffer.from(frame.samples), frame.sampleRate, frame.numChannels);
}

async function handlePushScreenShareFloat(args: unknown): Promise<boolean> {
	const session = activeSession;
	if (!session) return false;
	const frame = assertFloatPcmFrameArgs(args, 'push-screen-audio-float');
	return session.engine.pushScreenShareFloat(bufferFromPayload(frame.samples), frame.sampleRate, frame.numChannels);
}

async function handleUnpublishScreenShareAudio(): Promise<void> {
	const session = activeSession;
	if (!session) return;
	await session.engine.unpublishScreenShareAudio();
}

async function handleListAudioOutputDevices(): Promise<Array<VoiceEngineV2BridgeAudioOutputDevice>> {
	if (admState.status === 'warming') {
		startAdmWarmup();
		return [];
	}
	const engine = await ensureEngineReady();
	try {
		return await handleNativeVoiceEngineListAudioOutputDevices(engine);
	} catch (error) {
		if (isTransientAudioDeviceModuleError(error)) {
			revertAdmToWarming(error);
			return [];
		}
		throw error;
	}
}

async function handleListAudioInputDevices(): Promise<Array<VoiceEngineV2BridgeAudioInputDevice>> {
	if (admState.status === 'warming') {
		startAdmWarmup();
		return [];
	}
	const engine = await ensureEngineReady();
	try {
		return await handleNativeVoiceEngineListAudioInputDevices(engine);
	} catch (error) {
		if (isTransientAudioDeviceModuleError(error)) {
			revertAdmToWarming(error);
			return [];
		}
		throw error;
	}
}

async function handleSetAudioOutputDevice(deviceId: string): Promise<void> {
	if (admState.status === 'warming') {
		pendingOutputDeviceId = deviceId;
		pendingOutputDeviceAdmRetryAttempt = 0;
		startAdmWarmup();
		return;
	}
	const engine = await ensureEngineReady();
	try {
		await engine.setAudioOutputDevice(deviceId);
	} catch (error) {
		if (isTransientAudioDeviceModuleError(error)) {
			pendingOutputDeviceId = deviceId;
			pendingOutputDeviceAdmRetryAttempt = 0;
			revertAdmToWarming(error);
			return;
		}
		throw error;
	}
}

async function handleSetParticipantVolume(args: VoiceEngineV2BridgeParticipantVolumeOptions): Promise<void> {
	const session = activeSession;
	if (!session) return;
	await session.engine.setParticipantVolume(args.participantSid, clampVoiceEngineV2ParticipantVolume(args.volume));
}

async function handleSetRemoteTrackSubscription(
	args: VoiceEngineV2BridgeRemoteTrackSubscriptionOptions,
): Promise<void> {
	const session = activeSession;
	if (!session) return;
	if (typeof session.engine.setRemoteTrackSubscription !== 'function') {
		logger.warn('Native voice engine addon does not expose setRemoteTrackSubscription');
		return;
	}
	await session.engine.setRemoteTrackSubscription({
		participantIdentity: args.participantIdentity,
		source: args.source,
		subscribed: args.subscribed,
		enabled: args.enabled ?? args.subscribed,
		...(args.quality ? {quality: args.quality} : {}),
	});
}

function bufferFromPayload(payload: ArrayBuffer | ArrayBufferView): Buffer {
	if (payload instanceof ArrayBuffer) {
		return Buffer.from(payload);
	}
	return Buffer.from(payload.buffer, payload.byteOffset, payload.byteLength);
}

async function handlePublishData(args: VoiceEngineV2BridgePublishDataOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	if (typeof session.engine.publishData !== 'function') {
		throw new Error('Native voice engine addon does not expose publishData');
	}
	const topic = args.topic?.trim();
	const destinationIdentities = args.destinationIdentities
		?.map((identity) => identity.trim())
		.filter((identity) => identity.length > 0);
	await session.engine.publishData(
		bufferFromPayload(args.payload),
		args.reliable !== false,
		topic && topic.length > 0 ? topic : undefined,
		destinationIdentities && destinationIdentities.length > 0 ? destinationIdentities : undefined,
	);
}

async function handleListCameraDevices(): Promise<Array<VoiceEngineV2BridgeCameraDevice>> {
	const engine = await ensureEngineReady();
	return handleNativeVoiceEngineListCameraDevices(engine);
}

async function handlePublishCamera(opts: VoiceEngineV2BridgePublishCameraOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	await session.engine.publishCamera(opts);
}

async function handlePublishNativeCameraSink(
	opts: VoiceEngineV2BridgePublishCameraOptions,
): Promise<VoiceEngineV2BridgePublishNativeCameraSinkResult> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	if (typeof session.engine.publishNativeCameraSink !== 'function') {
		throw new Error('Native voice engine addon does not expose publishNativeCameraSink');
	}
	const result = await session.engine.publishNativeCameraSink(opts);
	if (!isVoiceEngineV2BridgePublishProcessedCameraResult(result)) {
		throw new Error('Native voice engine returned invalid publishNativeCameraSink result');
	}
	return result;
}

async function handlePublishProcessedCamera(
	opts: VoiceEngineV2BridgePublishProcessedCameraOptions,
): Promise<VoiceEngineV2BridgePublishProcessedCameraResult> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	const result = await session.engine.publishProcessedCamera(opts);
	if (!isVoiceEngineV2BridgePublishProcessedCameraResult(result)) {
		throw new Error('Native voice engine returned invalid publishProcessedCamera result');
	}
	return result;
}

async function handlePushProcessedCameraFrame(frame: VoiceEngineV2BridgeProcessedCameraFrame): Promise<boolean> {
	const session = activeSession;
	if (!session) return false;
	return session.engine.pushProcessedCameraFrame({
		format: frame.format,
		width: frame.width,
		height: frame.height,
		timestampUs: frame.timestampUs,
		data: bufferFromPayload(frame.data),
	});
}

function cameraBackgroundEngine(): NativeVoiceEngineInstance | null {
	if (activeSession) return activeSession.engine;
	if (previewSession) return engineSingleton;
	return null;
}

async function handlePushCameraBackgroundFrame(frame: VoiceEngineV2BridgeProcessedCameraFrame): Promise<boolean> {
	const engine = cameraBackgroundEngine();
	if (!engine) return false;
	return engine.pushCameraBackgroundFrame({
		format: frame.format,
		width: frame.width,
		height: frame.height,
		timestampUs: frame.timestampUs,
		data: bufferFromPayload(frame.data),
	});
}

function handleClearCameraBackgroundFrame(): void {
	const engine = cameraBackgroundEngine();
	if (!engine) return;
	engine.clearCameraBackgroundFrame();
}

async function handleUpdateCameraCapture(opts: VoiceEngineV2BridgeUpdateCameraCaptureOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	if (typeof session.engine.updateCameraCapture !== 'function') {
		throw new Error('Native voice engine addon does not expose updateCameraCapture');
	}
	await session.engine.updateCameraCapture(opts);
}

async function handlePublishDeviceScreenShare(opts: VoiceEngineV2BridgePublishDeviceScreenShareOptions): Promise<void> {
	const session = activeSession;
	if (!session) {
		throw new Error('Native voice engine is not connected');
	}
	await session.engine.publishDeviceScreenShare(opts);
}

async function handleUnpublishCamera(): Promise<void> {
	const session = activeSession;
	if (!session) return;
	try {
		await session.engine.unpublishCamera();
	} catch (error) {
		logger.warn('Native voice engine unpublishCamera failed', error);
		throw error;
	}
}

function handleIsPublishingCamera(): boolean {
	const session = activeSession;
	return session?.engine.isPublishingCamera() ?? false;
}

async function handleStartCameraPreview(
	sender: WebContents,
	opts: VoiceEngineV2BridgeStartCameraPreviewOptions,
): Promise<VoiceEngineV2BridgeCameraPreviewInfo> {
	const active = activeSession;
	if (active) {
		if (typeof active.engine.startCameraPreview !== 'function') {
			throw new Error('Native voice engine addon does not expose startCameraPreview');
		}
		const result = await active.engine.startCameraPreview(opts);
		if (!isVoiceEngineV2BridgeCameraPreviewInfo(result)) {
			throw new Error('Native voice engine returned invalid startCameraPreview result');
		}
		return result;
	}
	const engine = await ensureEngineReady();
	if (typeof engine.startCameraPreview !== 'function') {
		throw new Error('Native voice engine addon does not expose startCameraPreview');
	}
	stopPreviewSession();
	const session: PreviewSessionRecord = {sender, senderLifecycle: null};
	previewSession = session;
	registerPreviewVideoFrameForwarding(engine, session);
	session.senderLifecycle = attachWebContentsOwnerLifecycle(sender, (reason) => {
		if (previewSession !== session) return;
		logger.info('Native voice engine owner renderer invalidated; stopping standalone preview', {reason});
		stopPreviewSession();
	});
	try {
		const result = await engine.startCameraPreview(opts);
		if (!isVoiceEngineV2BridgeCameraPreviewInfo(result)) {
			throw new Error('Native voice engine returned invalid startCameraPreview result');
		}
		return result;
	} catch (error) {
		if (previewSession === session) {
			stopPreviewSession();
		}
		throw error;
	}
}

function handleStopCameraPreview(): void {
	const active = activeSession;
	if (active && typeof active.engine.stopCameraPreview === 'function') {
		try {
			active.engine.stopCameraPreview();
		} catch (error) {
			logger.warn('Native voice engine stopCameraPreview failed', error);
		}
	}
	stopPreviewSession();
}

async function handleGetConnectionStats(): Promise<VoiceEngineV2BridgeStats> {
	const session = activeSession;
	if (!session) {
		return {rttMs: null, outbound: [], inbound: []};
	}
	const stats = await session.engine.getConnectionStats();
	if (typeof session.engine.droppedVideoFrameCallbacks !== 'function') return stats;
	const droppedVideoFrameCallbacks = session.engine.droppedVideoFrameCallbacks();
	if (!Number.isFinite(droppedVideoFrameCallbacks)) return stats;
	return {
		...stats,
		droppedVideoFrameCallbacks: Math.max(0, Math.trunc(droppedVideoFrameCallbacks)),
	};
}

let handlersRegistered = false;

export function registerNativeVoiceEngineHandlers(): void {
	if (handlersRegistered) return;
	handlersRegistered = true;
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.isSupported, (): boolean => isNativeVoiceEngineSupported());
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.getCapabilities,
		(): VoiceEngineV2BridgeCapabilities => getVoiceEngineV2BridgeCapabilities(),
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.prewarm, async (): Promise<void> => {
		await prewarmNativeVoiceEngine();
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.getVoiceEngineReadiness,
		(): VoiceEngineV2BridgeReadiness => getVoiceEngineReadiness(),
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.getAudioDeviceModuleState,
		(): VoiceEngineV2BridgeAudioDeviceModuleState => {
			startAdmWarmup();
			return getAudioDeviceModuleState();
		},
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.getHardwareEncoderCapabilities,
		(): VoiceEngineV2BridgeHardwareEncoderCapabilities => getHardwareEncoderCapabilities(),
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.connect, async (event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgeConnectOptions(args)) {
			throw new Error('Invalid voice-engine connect args');
		}
		await handleConnect(event.sender, args);
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.disconnect, async (): Promise<void> => {
		await teardownActiveSession();
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.isConnected, (): boolean => {
		const session = activeSession;
		if (!session) return false;
		try {
			return session.engine.isConnected();
		} catch (error) {
			logger.warn('Native voice engine isConnected() threw', error);
			return false;
		}
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreen, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgePublishScreenOptions(args)) {
			throw new Error('Invalid voice-engine publish-screen args');
		}
		await handlePublishScreen(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.updateScreenShareEncoding,
		async (_event, args: unknown): Promise<void> => {
			if (!isVoiceEngineV2BridgeUpdateScreenShareEncodingOptions(args)) {
				throw new Error('Invalid voice-engine update-screen-share-encoding args');
			}
			await handleUpdateScreenShareEncoding(args);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.unpublishScreen, async (): Promise<void> => {
		await handleUnpublishScreen();
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.publishScreenAudio, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgePublishScreenAudioOptions(args)) {
			throw new Error('Invalid voice-engine publish-screen-audio args');
		}
		await handlePublishScreenShareAudio(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.pushScreenAudioPcm,
		async (_event, args: unknown): Promise<boolean> => handlePushScreenSharePcm(args),
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.pushScreenAudioFloat,
		async (_event, args: unknown): Promise<boolean> => handlePushScreenShareFloat(args),
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.unpublishScreenAudio, async (): Promise<void> => {
		await handleUnpublishScreenShareAudio();
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.setMicEnabled,
		async (_event, enabled: unknown): Promise<VoiceEngineV2BridgeOperationResult> => {
			if (typeof enabled !== 'boolean') {
				return voiceEngineOperationFailure('invalid-args', 'Invalid voice-engine set-mic-enabled args');
			}
			return handleSetMicEnabled(enabled);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.setSpeakingDetection, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgeSpeakingDetectionOptions(args)) {
			throw new Error('Invalid voice-engine set-speaking-detection args');
		}
		await handleSetSpeakingDetection(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.publishMicrophone,
		async (_event, args: unknown): Promise<VoiceEngineV2BridgeOperationResult> => {
			if (!isVoiceEngineV2BridgePublishMicrophoneOptions(args)) {
				return voiceEngineOperationFailure(
					'invalid-args',
					'Invalid voice-engine publish-microphone args',
					'microphoneCapture',
				);
			}
			return handlePublishMicrophone(args);
		},
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.pushPcm,
		async (_event, args: unknown): Promise<boolean> => handlePushPcm(args),
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.listAudioInputDevices,
		async (): Promise<Array<VoiceEngineV2BridgeAudioInputDevice>> => handleListAudioInputDevices(),
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.listAudioOutputDevices,
		async (): Promise<Array<VoiceEngineV2BridgeAudioOutputDevice>> => handleListAudioOutputDevices(),
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.setAudioOutputDevice,
		async (_event, deviceId: unknown): Promise<void> => {
			if (typeof deviceId !== 'string') {
				throw new Error('Invalid voice-engine set-audio-output-device args');
			}
			await handleSetAudioOutputDevice(deviceId);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.setParticipantVolume, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2ParticipantVolumeOptions(args)) {
			throw new Error('Invalid voice-engine set-participant-volume args');
		}
		await handleSetParticipantVolume(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.setRemoteTrackSubscription,
		async (_event, args: unknown): Promise<void> => {
			if (!isVoiceEngineV2BridgeRemoteTrackSubscriptionOptions(args)) {
				throw new Error('Invalid voice-engine set-remote-track-subscription args');
			}
			await handleSetRemoteTrackSubscription(args);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.publishData, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgePublishDataOptions(args)) {
			throw new Error('Invalid voice-engine publish-data args');
		}
		await handlePublishData(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.listCameraDevices,
		async (): Promise<Array<VoiceEngineV2BridgeCameraDevice>> => handleListCameraDevices(),
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.publishCamera, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgePublishCameraOptions(args)) {
			throw new Error('Invalid voice-engine publish-camera args');
		}
		await handlePublishCamera(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.publishNativeCameraSink,
		async (_event, args: unknown): Promise<VoiceEngineV2BridgePublishNativeCameraSinkResult> => {
			if (!isVoiceEngineV2BridgePublishCameraOptions(args)) {
				throw new Error('Invalid voice-engine publish-native-camera-sink args');
			}
			return handlePublishNativeCameraSink(args);
		},
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.publishProcessedCamera,
		async (_event, args: unknown): Promise<VoiceEngineV2BridgePublishProcessedCameraResult> => {
			if (!isVoiceEngineV2BridgePublishProcessedCameraOptions(args)) {
				throw new Error('Invalid voice-engine publish-processed-camera args');
			}
			return handlePublishProcessedCamera(args);
		},
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.pushProcessedCameraFrame,
		async (_event, args: unknown): Promise<boolean> => {
			if (!isVoiceEngineV2BridgeProcessedCameraFrame(args)) {
				throw new Error('Invalid voice-engine push-processed-camera-frame args');
			}
			return handlePushProcessedCameraFrame(args);
		},
	);
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.pushCameraBackgroundFrame,
		async (_event, args: unknown): Promise<boolean> => {
			if (!isVoiceEngineV2BridgeProcessedCameraFrame(args)) {
				throw new Error('Invalid voice-engine push-camera-background-frame args');
			}
			return handlePushCameraBackgroundFrame(args);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.clearCameraBackgroundFrame, async (): Promise<void> => {
		handleClearCameraBackgroundFrame();
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.updateCameraCapture, async (_event, args: unknown): Promise<void> => {
		if (!isVoiceEngineV2BridgePublishCameraOptions(args)) {
			throw new Error('Invalid voice-engine update-camera-capture args');
		}
		await handleUpdateCameraCapture(args);
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.publishDeviceScreenShare,
		async (_event, args: unknown): Promise<void> => {
			if (!isVoiceEngineV2BridgePublishDeviceScreenShareOptions(args)) {
				throw new Error('Invalid voice-engine publish-device-screen-share args');
			}
			await handlePublishDeviceScreenShare(args);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.unpublishCamera, async (): Promise<void> => {
		await handleUnpublishCamera();
	});
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.isPublishingCamera, (): boolean => handleIsPublishingCamera());
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.startCameraPreview,
		async (event, args: unknown): Promise<VoiceEngineV2BridgeCameraPreviewInfo> => {
			if (!isVoiceEngineV2BridgePublishCameraOptions(args)) {
				throw new Error('Invalid voice-engine start-camera-preview args');
			}
			return handleStartCameraPreview(event.sender, args);
		},
	);
	ipcMain.handle(VOICE_ENGINE_V2_IPC_CHANNELS.stopCameraPreview, async (): Promise<void> => {
		handleStopCameraPreview();
	});
	ipcMain.handle(
		VOICE_ENGINE_V2_IPC_CHANNELS.getConnectionStats,
		async (): Promise<VoiceEngineV2BridgeStats> => handleGetConnectionStats(),
	);
}

export function cleanupNativeVoiceEngine(): Promise<void> {
	if (!handlersRegistered) return Promise.resolve();
	for (const channel of Object.values(VOICE_ENGINE_V2_IPC_CHANNELS)) {
		ipcMain.removeHandler(channel);
	}
	handlersRegistered = false;
	stopPreviewSession();
	return teardownActiveSession();
}
