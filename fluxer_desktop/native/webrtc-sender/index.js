// SPDX-License-Identifier: AGPL-3.0-or-later

const {existsSync} = require('node:fs');
const {join, sep} = require('node:path');

const MODULE_NAME = '@fluxer/webrtc-sender';

function resolveNativeRoot() {
	const asarSegment = `${sep}app.asar${sep}`;
	if (!__dirname.includes(asarSegment)) return __dirname;
	const unpackedDir = __dirname.replace(asarSegment, `${sep}app.asar.unpacked${sep}`);
	return existsSync(unpackedDir) ? unpackedDir : __dirname;
}

function nativeFileName(platform = process.platform, arch = process.arch) {
	if ((arch !== 'x64' && arch !== 'arm64') || !['darwin', 'linux', 'win32'].includes(platform)) {
		throw new Error(`${MODULE_NAME} not supported on ${platform}-${arch}`);
	}
	if (platform === 'darwin') return `webrtc-sender.darwin-${arch}.node`;
	if (platform === 'linux') return `webrtc-sender.linux-${arch}-gnu.node`;
	if (platform === 'win32') return `webrtc-sender.win32-${arch}-msvc.node`;
	throw new Error(`${MODULE_NAME} not supported on ${platform}-${arch}`);
}

let binding = null;
let loadError = null;

try {
	const nativePath = join(resolveNativeRoot(), nativeFileName());
	if (existsSync(nativePath)) {
		try {
			binding = require(nativePath);
		} catch (error) {
			loadError = error instanceof Error ? error : new Error(String(error));
		}
	} else {
		loadError = new Error(`${MODULE_NAME} native binary missing: ${nativePath}`);
	}
} catch (error) {
	loadError = error instanceof Error ? error : new Error(String(error));
}

function isSupported() {
	return Boolean(binding);
}

function unavailableHardwareEncoderCapability(reason, detail) {
	return {
		available: false,
		backend: 'none',
		compiled: false,
		runtime: false,
		codecs: [],
		zeroCopy: false,
		nativeInputs: [],
		reason,
		detail,
	};
}

function getHardwareEncoderCapability() {
	if (!binding) {
		return unavailableHardwareEncoderCapability(
			'native_binding_unavailable',
			loadError ? loadError.message : `${MODULE_NAME} binding unavailable`,
		);
	}
	if (typeof binding.getHardwareEncoderCapability !== 'function') {
		return unavailableHardwareEncoderCapability(
			'native_capability_unavailable',
			`${MODULE_NAME} native binding does not export getHardwareEncoderCapability`,
		);
	}
	return binding.getHardwareEncoderCapability();
}

function getHardwareEncoderCapabilities() {
	return getHardwareEncoderCapability();
}

function hasVoiceEngineMethod(name) {
	const prototype = binding?.VoiceEngine?.prototype;
	return Boolean(prototype && typeof prototype[name] === 'function');
}

function hasNativeCameraBackgrounds() {
	if (!binding) return false;
	if (typeof binding.hasNativeCameraBackgrounds !== 'function') return false;
	return binding.hasNativeCameraBackgrounds() === true;
}

function getCapabilities() {
	const hasVoiceEngine = Boolean(binding && typeof binding.VoiceEngine === 'function');
	return {
		microphoneCapture: hasVoiceEngineMethod('publishDeviceMicrophone'),
		syntheticMicrophonePcm: hasVoiceEngineMethod('publishMicrophone'),
		cameraCapture: hasVoiceEngineMethod('publishCamera') && hasVoiceEngineMethod('listCameraDevices'),
		nativeCameraBackgrounds: hasNativeCameraBackgrounds(),
		screenShare: hasVoiceEngineMethod('publishScreenShare') && hasVoiceEngineMethod('unpublishScreenShare'),
		screenShareEncodingUpdate: hasVoiceEngineMethod('updateScreenShareEncoding'),
		screenShareAudio:
			hasVoiceEngineMethod('publishScreenShareAudio') &&
			hasVoiceEngineMethod('pushScreenSharePcm') &&
			hasVoiceEngineMethod('pushScreenShareFloat') &&
			hasVoiceEngineMethod('unpublishScreenShareAudio'),
		deviceLists: hasVoiceEngineMethod('listAudioInputDevices') && hasVoiceEngineMethod('listAudioOutputDevices'),
		outputDeviceSelection: hasVoiceEngineMethod('setAudioOutputDevice'),
		participantVolume: hasVoiceEngineMethod('setParticipantVolume'),
		remoteTrackSubscription: hasVoiceEngineMethod('setRemoteTrackSubscription'),
		dataChannel: hasVoiceEngineMethod('publishData'),
		connectionStats: hasVoiceEngineMethod('getConnectionStats'),
		nativeVideoFrames: hasVoiceEngineMethod('setVideoFrameCallback'),
		hardwareEncoderCapabilities: hasVoiceEngine && typeof getHardwareEncoderCapability === 'function',
	};
}

function requireScreenShareCaptureId(options, operation) {
	if (!options || typeof options.captureId !== 'string' || options.captureId.trim().length === 0) {
		throw new Error(`${operation} requires a non-empty captureId`);
	}
}

function getEngineBridgeVersion() {
	if (!binding) return null;
	if (typeof binding.getEngineBridgeVersion !== 'function') return null;
	return binding.getEngineBridgeVersion();
}

function assertEngineBridgeVersion(version) {
	if (!binding) {
		throw loadError || new Error(`${MODULE_NAME} binding unavailable`);
	}
	if (typeof binding.assertEngineBridgeVersion !== 'function') {
		throw new Error(`${MODULE_NAME} native binding does not export assertEngineBridgeVersion`);
	}
	binding.assertEngineBridgeVersion(version);
}

function prewarmVoiceEngine() {
	if (!binding) {
		if (loadError) throw loadError;
		return;
	}
	if (typeof binding.prewarmVoiceEngine === 'function') {
		return binding.prewarmVoiceEngine();
	}
}

function probeAudioDeviceModule() {
	if (!binding) {
		if (loadError) throw loadError;
		return Promise.resolve(false);
	}
	if (typeof binding.probeAudioDeviceModule === 'function') {
		return Promise.resolve(binding.probeAudioDeviceModule());
	}
	return Promise.resolve(true);
}

function normalizeCameraOptions(opts = {}) {
	return {
		deviceId: opts.deviceId,
		width: opts.width,
		height: opts.height,
		frameRate: opts.frameRate,
		mirror: opts.mirror,
		backgroundMode: opts.backgroundMode,
		backgroundCustomMediaPath: opts.backgroundCustomMediaPath,
		backgroundCustomMediaKind: opts.backgroundCustomMediaKind,
		backgroundBlurStrength: opts.backgroundBlurStrength,
		codec: opts.codec,
		maxBitrateBps: opts.maxBitrateBps,
		maxFramerate: opts.maxFramerate,
	};
}

class VoiceEngine {
	constructor() {
		if (!binding) {
			throw loadError || new Error(`${MODULE_NAME} binding unavailable`);
		}
		this.native = new binding.VoiceEngine();
	}

	setEventCallback(callback) {
		return this.native.setEventCallback(callback);
	}

	setVideoFrameCallback(callback) {
		return this.native.setVideoFrameCallback(callback);
	}

	clearVideoFrameCallback() {
		if (typeof this.native.clearVideoFrameCallback === 'function') {
			return this.native.clearVideoFrameCallback();
		}
		if (typeof this.native.setVideoFrameCallback === 'function') {
			return this.native.setVideoFrameCallback(() => {});
		}
		return undefined;
	}

	setCountInboundAudio(enabled) {
		return this.native.setCountInboundAudio(enabled);
	}

	connect(url, token, e2eeKey, options) {
		return this.native.connect(url, token, e2eeKey, options);
	}

	disconnect() {
		return this.native.disconnect();
	}

	isConnected() {
		return this.native.isConnected();
	}

	publishScreenShare(width, height, codec = '', maxBitrateBps, maxFramerate, simulcast, options) {
		requireScreenShareCaptureId(options, 'Native screen-share publish');
		return this.native.publishScreenShare(width, height, codec, maxBitrateBps, maxFramerate, simulcast, options);
	}

	updateScreenShareEncoding(width, height, maxBitrateBps, maxFramerate, options) {
		requireScreenShareCaptureId(options, 'Native screen-share encoding update');
		if (typeof this.native.updateScreenShareEncoding === 'function') {
			return this.native.updateScreenShareEncoding(width, height, maxBitrateBps, maxFramerate, options);
		}
		return Promise.reject(new Error('Native screen-share encoding update is unavailable'));
	}

	createScreenFrameSinkHandle(captureId) {
		if (typeof this.native.createScreenFrameSinkHandle !== 'function') return null;
		return this.native.createScreenFrameSinkHandle(captureId);
	}

	createScreenAudioSinkHandle() {
		if (typeof this.native.createScreenAudioSinkHandle !== 'function') return null;
		return this.native.createScreenAudioSinkHandle();
	}

	unpublishScreenShare() {
		return this.native.unpublishScreenShare();
	}

	isPublishingScreen() {
		return this.native.isPublishingScreen();
	}

	publishScreenShareAudio(sampleRate, numChannels) {
		return this.native.publishScreenShareAudio(sampleRate, numChannels);
	}

	pushScreenSharePcm(buffer, sampleRate, numChannels) {
		return this.native.pushScreenSharePcm(buffer, sampleRate, numChannels);
	}

	pushScreenShareFloat(buffer, sampleRate, numChannels) {
		return this.native.pushScreenShareFloat(buffer, sampleRate, numChannels);
	}

	unpublishScreenShareAudio() {
		return this.native.unpublishScreenShareAudio();
	}

	isPublishingScreenAudio() {
		return this.native.isPublishingScreenAudio();
	}

	publishMicrophone(sampleRate, numChannels) {
		return this.native.publishMicrophone(sampleRate, numChannels);
	}

	publishDeviceMicrophone(opts = {}) {
		if (typeof this.native.publishDeviceMicrophone !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export publishDeviceMicrophone`);
		}
		return this.native.publishDeviceMicrophone({
			deviceId: opts.deviceId,
			echoCancellation: opts.echoCancellation,
			noiseSuppression: opts.noiseSuppression,
			autoGainControl: opts.autoGainControl,
			...(opts.deepFilter !== undefined ? {deepFilter: opts.deepFilter} : {}),
			...(opts.deepFilterNoiseReductionLevel !== undefined
				? {deepFilterNoiseReductionLevel: opts.deepFilterNoiseReductionLevel}
				: {}),
			maxBitrateBps: opts.maxBitrateBps,
		});
	}

	pushPcm(buffer, sampleRate, numChannels) {
		return this.native.pushPcm(buffer, sampleRate, numChannels);
	}

	setMicEnabled(enabled) {
		return this.native.setMicEnabled(enabled);
	}

	setSpeakingDetection(localThresholdRms, remoteThresholdRms) {
		if (typeof this.native.setSpeakingDetection !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export setSpeakingDetection`);
		}
		return this.native.setSpeakingDetection(localThresholdRms, remoteThresholdRms);
	}

	publishCamera(opts = {}) {
		return this.native.publishCamera(normalizeCameraOptions(opts));
	}

	updateCameraCapture(opts = {}) {
		if (typeof this.native.updateCameraCapture !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export updateCameraCapture`);
		}
		return this.native.updateCameraCapture(normalizeCameraOptions(opts));
	}

	publishProcessedCamera(opts) {
		if (typeof this.native.publishProcessedCamera !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export publishProcessedCamera`);
		}
		return this.native.publishProcessedCamera({
			width: opts.width,
			height: opts.height,
			frameRate: opts.frameRate,
		});
	}

	publishNativeCameraSink(opts = {}) {
		if (typeof this.native.publishNativeCameraSink !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export publishNativeCameraSink`);
		}
		return this.native.publishNativeCameraSink(normalizeCameraOptions(opts));
	}

	createCameraFrameSinkHandle() {
		if (typeof this.native.createCameraFrameSinkHandle !== 'function') return null;
		return this.native.createCameraFrameSinkHandle();
	}

	pushProcessedCameraFrame(frame) {
		if (typeof this.native.pushProcessedCameraFrame !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export pushProcessedCameraFrame`);
		}
		return this.native.pushProcessedCameraFrame({
			format: frame.format,
			width: frame.width,
			height: frame.height,
			timestampUs: frame.timestampUs,
			data: frame.data,
		});
	}

	startCameraPreview(opts = {}) {
		if (typeof this.native.startCameraPreview !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export startCameraPreview`);
		}
		return this.native.startCameraPreview(normalizeCameraOptions(opts));
	}

	stopCameraPreview() {
		if (typeof this.native.stopCameraPreview !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export stopCameraPreview`);
		}
		return this.native.stopCameraPreview();
	}

	pushCameraBackgroundFrame(frame) {
		if (typeof this.native.pushCameraBackgroundFrame !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export pushCameraBackgroundFrame`);
		}
		return this.native.pushCameraBackgroundFrame({
			format: frame.format,
			width: frame.width,
			height: frame.height,
			timestampUs: frame.timestampUs,
			data: frame.data,
		});
	}

	clearCameraBackgroundFrame() {
		if (typeof this.native.clearCameraBackgroundFrame !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export clearCameraBackgroundFrame`);
		}
		return this.native.clearCameraBackgroundFrame();
	}

	publishDeviceScreenShare(opts = {}) {
		if (typeof this.native.publishDeviceScreenShare !== 'function') {
			throw new Error(`${MODULE_NAME} native binding does not export publishDeviceScreenShare`);
		}
		return this.native.publishDeviceScreenShare(normalizeCameraOptions(opts));
	}

	listCameraDevices() {
		return this.native.listCameraDevices();
	}

	unpublishCamera() {
		return this.native.unpublishCamera();
	}

	isPublishingCamera() {
		return this.native.isPublishingCamera();
	}

	async listAudioInputDevices() {
		const json = await this.native.listAudioInputDevices();
		return JSON.parse(json);
	}

	droppedEngineEvents() {
		if (typeof this.native.droppedEngineEvents !== 'function') return 0;
		return this.native.droppedEngineEvents();
	}

	async listAudioOutputDevices() {
		const json = await this.native.listAudioOutputDevices();
		return JSON.parse(json);
	}

	setAudioOutputDevice(deviceId) {
		return this.native.setAudioOutputDevice(deviceId || '');
	}

	ensurePlatformAudio() {
		if (typeof this.native.ensurePlatformAudio !== 'function') return Promise.resolve();
		return this.native.ensurePlatformAudio();
	}

	setParticipantVolume(participantSid, volume) {
		return this.native.setParticipantVolume(participantSid, volume);
	}

	setRemoteTrackSubscription(opts = {}) {
		if (typeof this.native.setRemoteTrackSubscription !== 'function') return Promise.resolve();
		return this.native.setRemoteTrackSubscription(
			opts.participantIdentity || '',
			opts.source || '',
			opts.subscribed === true,
			opts.enabled !== false,
			opts.quality || undefined,
		);
	}

	publishData(payload, opts = {}) {
		if (typeof this.native.publishData !== 'function') {
			return Promise.reject(new Error(`${MODULE_NAME} native binding does not export publishData`));
		}
		const buffer = Buffer.isBuffer(payload)
			? payload
			: payload instanceof ArrayBuffer
				? Buffer.from(payload)
				: ArrayBuffer.isView(payload)
					? Buffer.from(payload.buffer, payload.byteOffset, payload.byteLength)
					: null;
		if (!buffer) {
			return Promise.reject(new TypeError('publishData payload must be a Buffer, ArrayBuffer, or typed array'));
		}
		return this.native.publishData(
			buffer,
			opts.reliable !== false,
			typeof opts.topic === 'string' ? opts.topic : undefined,
			Array.isArray(opts.destinationIdentities) ? opts.destinationIdentities : undefined,
		);
	}

	getConnectionStats() {
		const json = this.native.getConnectionStats();
		try {
			return JSON.parse(json);
		} catch {
			return {rttMs: null, outbound: [], inbound: []};
		}
	}

	inboundAudioFrames() {
		return this.native.inboundAudioFrames();
	}

	inboundVideoFrames() {
		return this.native.inboundVideoFrames();
	}

	droppedVideoFrameCallbacks() {
		if (typeof this.native.droppedVideoFrameCallbacks !== 'function') return 0;
		return this.native.droppedVideoFrameCallbacks();
	}
}

module.exports = {
	isSupported,
	getEngineBridgeVersion,
	assertEngineBridgeVersion,
	getHardwareEncoderCapability,
	getHardwareEncoderCapabilities,
	getCapabilities,
	hasNativeCameraBackgrounds,
	prewarmVoiceEngine,
	probeAudioDeviceModule,
	VoiceEngine,
	get loadError() {
		return loadError;
	},
	__nativeFileNameForTests: nativeFileName,
	__setBindingForTests(next) {
		binding = next;
		loadError = null;
	},
};
