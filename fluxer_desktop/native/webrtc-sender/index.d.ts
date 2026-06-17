// SPDX-License-Identifier: AGPL-3.0-or-later

export interface AudioOutputDevice {
	deviceId: string;
	label: string;
	isDefault: boolean;
}

export interface AudioInputDevice {
	deviceId: string;
	label: string;
	isDefault: boolean;
}

export interface PublishMicrophoneOptions {
	deviceId?: string;
	echoCancellation?: boolean;
	noiseSuppression?: boolean;
	autoGainControl?: boolean;
	deepFilter?: boolean;
	deepFilterNoiseReductionLevel?: number;
	maxBitrateBps?: number;
}

export interface VoiceEngineV2BridgeCapabilities {
	microphoneCapture: boolean;
	syntheticMicrophonePcm: boolean;
	cameraCapture: boolean;
	nativeCameraBackgrounds: boolean;
	screenShare: boolean;
	screenShareEncodingUpdate: boolean;
	screenShareAudio: boolean;
	deviceLists: boolean;
	outputDeviceSelection: boolean;
	participantVolume: boolean;
	remoteTrackSubscription: boolean;
	dataChannel: boolean;
	connectionStats: boolean;
	nativeVideoFrames: boolean;
	hardwareEncoderCapabilities: boolean;
}

export type VoiceEngineTrackKind = 'audio' | 'video';
export type VoiceEngineTrackSource =
	| 'unknown'
	| 'camera'
	| 'microphone'
	| 'screen_share'
	| 'screen_share_audio'
	| 'screenshare'
	| 'screenshareAudio';
export type VoiceEngineSubscriptionStatus = 'desired' | 'subscribed' | 'unsubscribed';
export type VoiceEngineConnectionQuality = 'excellent' | 'good' | 'poor' | 'lost';

export interface VoiceEngineParticipantEventPayload {
	sid: string;
	identity: string;
	name: string;
}

export interface VoiceEngineTrackEventPayload {
	participantSid: string;
	identity: string;
	participantName: string;
	trackSid: string;
	trackName: string;
	kind: VoiceEngineTrackKind;
	source: VoiceEngineTrackSource;
	muted: boolean;
}

export interface VoiceEngineSubscribedTrackEventPayload extends VoiceEngineTrackEventPayload {
	subscribed: boolean;
	subscriptionStatus: VoiceEngineSubscriptionStatus;
}

export interface VoiceEngineTrackSubscriptionFailedEventPayload {
	participantSid: string;
	identity: string;
	participantName: string;
	trackSid: string;
	error: string;
	trackName?: string;
	kind?: VoiceEngineTrackKind;
	source?: VoiceEngineTrackSource;
	muted?: boolean;
	subscribed?: boolean;
	subscriptionStatus?: VoiceEngineSubscriptionStatus;
}

export interface VoiceEngineLocalTrackRepublishedEventPayload extends VoiceEngineTrackEventPayload {
	previousTrackSid: string;
}

export interface VoiceEngineV2BridgeEventPayloads {
	connected: Record<keyof any, never>;
	connectionState: {state: string};
	disconnected: {reason: string};
	participantJoined: VoiceEngineParticipantEventPayload;
	participantLeft: VoiceEngineParticipantEventPayload;
	participantNameChanged: {sid: string; identity: string; oldName: string; name: string};
	participantMetadataChanged: {
		sid: string;
		identity: string;
		name: string;
		oldMetadata: string;
		metadata: string;
		attributes: Record<string, string>;
	};
	participantAttributesChanged: {
		sid: string;
		identity: string;
		name: string;
		attributes: Record<string, string>;
		changedAttributes: Record<string, string>;
	};
	trackPublished: VoiceEngineSubscribedTrackEventPayload;
	trackUnpublished: VoiceEngineSubscribedTrackEventPayload;
	trackSubscribed: VoiceEngineSubscribedTrackEventPayload;
	trackUnsubscribed: VoiceEngineSubscribedTrackEventPayload;
	trackSubscriptionFailed: VoiceEngineTrackSubscriptionFailedEventPayload;
	trackMuted: VoiceEngineTrackEventPayload;
	trackUnmuted: VoiceEngineTrackEventPayload;
	localTrackPublished: VoiceEngineTrackEventPayload;
	localTrackUnpublished: VoiceEngineTrackEventPayload;
	localTrackRepublished: VoiceEngineLocalTrackRepublishedEventPayload;
	activeSpeakers: {sids: Array<string>; participants: Array<VoiceEngineParticipantEventPayload>};
	connectionQuality: {sid: string; identity: string; name: string; quality: VoiceEngineConnectionQuality};
	dataReceived: {
		payloadBytes: Array<number>;
		payloadText?: string;
		topic?: string;
		reliable: boolean;
		kind: 'reliable' | 'lossy';
		participantSid?: string;
		identity?: string;
		participantName?: string;
	};
	e2eeState: {sid: string; identity: string; name: string; state: string};
	stats: VoiceEngineV2BridgeStats;
	audioPlaybackUnavailable: {message: string};
}

export type VoiceEngineKnownEventType = keyof VoiceEngineV2BridgeEventPayloads;
export type VoiceEngineV2BridgeEventType = VoiceEngineKnownEventType | (string & {});

export interface VoiceEngineOutboundStats {
	trackSid: string;
	source: string;
	kind: VoiceEngineTrackKind;
	codec?: string;
	bitrateKbps: number;
	packetsLost: number;
	fps?: number;
}

export interface VoiceEngineInboundStats {
	participantSid: string;
	trackSid: string;
	kind: VoiceEngineTrackKind;
	codec?: string;
	bitrateKbps: number;
	packetsLost: number;
	jitterMs?: number;
	audioLevel?: number;
}

export interface VoiceEngineV2BridgeStats {
	rttMs: number | null;
	outbound: Array<VoiceEngineOutboundStats>;
	inbound: Array<VoiceEngineInboundStats>;
	droppedVideoFrameCallbacks?: number;
	send?: VoiceEngineSendStats | null;
}

export interface VoiceEngineSendStats {
	outgoingVideoQueueDepth: number;
	outgoingVideoFramesProduced: number;
	outgoingVideoFramesAccepted: number;
	outgoingVideoFramesDropped: number;
	outgoingVideoFramesCoalesced: number;
	outgoingVideoFramesCaptured: number;
	outgoingVideoCaptureFailures: number;
	outgoingVideoEffectiveFps: number;
	outgoingVideoTargetFps: number;
	outgoingVideoMaxQueueAgeMs: number;
	outgoingVideoMaxPushLatencyMs: number;
	outgoingAudioBufferTargetMs: number;
	outgoingAudioBufferMaxMs: number;
	outgoingAudioUnderruns: number;
	outgoingAudioRebuffers: number;
	outgoingAudioMaxFrameGapMs: number;
	adaptiveSendTier: string;
	adaptiveSendReason: string;
}

export interface PublishScreenShareOptions {
	adaptiveSend?: boolean;
	minVideoFps?: number;
	maxAudioBufferMs?: number;
	pacing?: 'sender' | 'source';
	captureId: string;
	trackName?: string;
}

export type VoiceEngineRemoteTrackSubscriptionQuality = 'low' | 'medium' | 'high';

export interface VoiceEngineV2BridgeRemoteTrackSubscriptionOptions {
	participantIdentity: string;
	source: string;
	subscribed: boolean;
	enabled?: boolean;
	quality?: VoiceEngineRemoteTrackSubscriptionQuality;
}

export interface PublishCameraOptions {
	deviceId?: string;
	width?: number;
	height?: number;
	frameRate?: number;
	mirror?: boolean;
	backgroundMode?: 'none' | 'non' | 'blur' | 'custom';
	backgroundCustomMediaPath?: string;
	backgroundCustomMediaKind?: 'static' | 'animated' | 'video';
	backgroundBlurStrength?: number;
	codec?: '' | 'vp8' | 'vp9' | 'h264' | 'av1' | 'h265' | 'hevc';
	maxBitrateBps?: number;
	maxFramerate?: number;
}

export interface PublishProcessedCameraOptions {
	width: number;
	height: number;
	frameRate: number;
}

export interface PublishProcessedCameraResult {
	trackSid: string;
}

export type NativeCameraFrameSinkHandle = object;

export interface ProcessedCameraFrame {
	format: 'i420';
	width: number;
	height: number;
	timestampUs: number;
	data: Buffer;
}

export interface CameraDeviceInfo {
	deviceId: string;
	label: string;
	description: string;
	index?: number | null;
	deviceIdAliases: Array<string>;
}

export interface HardwareEncoderCapability {
	available: boolean;
	backend: 'nvenc' | 'videotoolbox' | 'none';
	compiled: boolean;
	runtime: boolean;
	codecs: Array<string>;
	zeroCopy: boolean;
	nativeInputs: Array<'dmabuf' | 'd3d11-texture' | string>;
	reason?: string;
	detail?: string;
}

export interface VoiceEngineV2BridgeConnectOptions {
	autoSubscribe?: boolean;
	adaptiveStream?: boolean;
	dynacast?: boolean;
}

export declare class VoiceEngine {
	constructor();

	setEventCallback(callback: (eventType: VoiceEngineV2BridgeEventType, jsonPayload: string) => void): void;
	setVideoFrameCallback(callback: (metaJson: string, data: Buffer) => void): void;
	clearVideoFrameCallback(): void;
	setCountInboundAudio(enabled: boolean): void;

	connect(url: string, token: string, e2eeKey?: Buffer, options?: VoiceEngineV2BridgeConnectOptions): Promise<void>;
	disconnect(): Promise<void>;
	isConnected(): boolean;

	publishScreenShare(
		width: number,
		height: number,
		codec: '' | 'vp8' | 'vp9' | 'h264' | 'av1' | 'h265' | 'hevc' | undefined,
		maxBitrateBps: number | undefined,
		maxFramerate: number | undefined,
		simulcast: boolean | undefined,
		options: PublishScreenShareOptions,
	): Promise<void>;
	updateScreenShareEncoding(
		width: number,
		height: number,
		maxBitrateBps: number | undefined,
		maxFramerate: number | undefined,
		options: PublishScreenShareOptions,
	): Promise<void>;
	createScreenFrameSinkHandle(captureId: string): unknown | null;
	unpublishScreenShare(): Promise<void>;
	isPublishingScreen(): boolean;

	publishScreenShareAudio(sampleRate: number, numChannels: number): Promise<void>;
	pushScreenSharePcm(buffer: Buffer, sampleRate: number, numChannels: number): Promise<boolean>;
	pushScreenShareFloat(buffer: Buffer, sampleRate: number, numChannels: number): Promise<boolean>;
	unpublishScreenShareAudio(): Promise<void>;
	isPublishingScreenAudio(): boolean;

	publishMicrophone(sampleRate: number, numChannels: number): Promise<void>;
	publishDeviceMicrophone(opts?: PublishMicrophoneOptions): Promise<void>;
	pushPcm(buffer: Buffer, sampleRate: number, numChannels: number): Promise<boolean>;
	setMicEnabled(enabled: boolean): Promise<void>;
	setSpeakingDetection(localThresholdRms: number, remoteThresholdRms: number): void;

	publishCamera(opts?: PublishCameraOptions): Promise<void>;
	updateCameraCapture(opts?: PublishCameraOptions): Promise<void>;
	publishProcessedCamera(opts: PublishProcessedCameraOptions): Promise<PublishProcessedCameraResult>;
	publishNativeCameraSink(opts?: PublishCameraOptions): Promise<PublishProcessedCameraResult>;
	createCameraFrameSinkHandle(): NativeCameraFrameSinkHandle | null;
	pushProcessedCameraFrame(frame: ProcessedCameraFrame): Promise<boolean>;
	publishDeviceScreenShare(opts?: PublishCameraOptions): Promise<void>;
	listCameraDevices(): Array<CameraDeviceInfo>;
	unpublishCamera(): Promise<void>;
	isPublishingCamera(): boolean;

	listAudioInputDevices(): Promise<Array<AudioInputDevice>>;
	listAudioOutputDevices(): Promise<Array<AudioOutputDevice>>;
	setAudioOutputDevice(deviceId: string): Promise<void>;
	ensurePlatformAudio(): Promise<void>;

	setParticipantVolume(participantSid: string, volume: number): Promise<void>;
	setRemoteTrackSubscription(options: VoiceEngineV2BridgeRemoteTrackSubscriptionOptions): Promise<void>;
	publishData(
		payload: Buffer | ArrayBuffer | Uint8Array,
		options?: {reliable?: boolean; topic?: string; destinationIdentities?: Array<string>},
	): Promise<void>;

	getConnectionStats(): Promise<VoiceEngineV2BridgeStats>;

	inboundAudioFrames(): number;
	inboundVideoFrames(): number;
	droppedVideoFrameCallbacks(): number;
	droppedEngineEvents(): number;
}

export declare function isSupported(): boolean;
export declare function getEngineBridgeVersion(): number | null;
export declare function assertEngineBridgeVersion(version: number): void;
export declare function getHardwareEncoderCapability(): HardwareEncoderCapability;
export declare function getHardwareEncoderCapabilities(): HardwareEncoderCapability;
export declare function getCapabilities(): VoiceEngineV2BridgeCapabilities;
export declare function hasNativeCameraBackgrounds(): boolean;
export declare function prewarmVoiceEngine(): void;
export declare function probeAudioDeviceModule(): Promise<boolean>;
export declare const loadError: Error | null;
export declare function __nativeFileNameForTests(platform: string, arch: string): string;
export declare function __setBindingForTests(binding: unknown): void;
