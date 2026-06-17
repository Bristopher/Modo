// SPDX-License-Identifier: AGPL-3.0-or-later

import {Logger} from '@app/features/platform/utils/AppLogger';
import {clearDesktopSourceIntent, setDesktopSourceIntent} from '@app/features/voice/state/DesktopSourceIntent';
import type {NativeScreenBridgeHandle} from '@app/features/voice/utils/native_screen_capture_bridge/shared';

const logger = new Logger('NativeScreenCaptureChromiumPreviewBridge');

interface ChromiumDesktopCaptureConstraints {
	mandatory: {
		chromeMediaSource: 'desktop';
		chromeMediaSourceId: string;
		maxWidth?: number;
		maxHeight?: number;
		maxFrameRate?: number;
	};
}

interface ChromiumPreviewBridgeOptions {
	maxWidth?: number;
	maxHeight?: number;
	maxFrameRate?: number;
	pauseWhenUnfocused?: boolean;
	previewPlatform?: NodeJS.Platform;
}

type DisplayMediaCursorPreviewConstraints = MediaTrackConstraints & {
	cursor?: 'always' | 'motion' | 'never';
};

export function shouldUseCursorHiddenDisplayMediaPreview(platform: NodeJS.Platform | undefined): boolean {
	return platform === 'darwin';
}

function resolvePreviewPlatform(options: ChromiumPreviewBridgeOptions): NodeJS.Platform | undefined {
	if (options.previewPlatform) return options.previewPlatform;
	return typeof process !== 'undefined' ? process.platform : undefined;
}

function stopAllTracks(stream: MediaStream): void {
	for (const t of stream.getTracks()) {
		try {
			t.stop();
		} catch {}
	}
}

function attachPreviewTrackLifecycle(stream: MediaStream, pauseWhenUnfocused: boolean): NativeScreenBridgeHandle {
	const track = stream.getVideoTracks()[0];
	if (!track) {
		stopAllTracks(stream);
		throw new Error('Chromium desktop capture returned no video track');
	}

	let cleanedUp = false;

	const applyEnabled = (enabled: boolean): void => {
		if (cleanedUp) return;
		if (track.enabled !== enabled) {
			track.enabled = enabled;
		}
	};
	const computeEnabled = (): boolean => {
		if (typeof document === 'undefined') return true;
		const visible = document.visibilityState !== 'hidden';
		const focused = document.hasFocus === undefined ? true : document.hasFocus();
		return visible && focused;
	};
	const handleVisibilityOrFocusChange = (): void => {
		if (!pauseWhenUnfocused) return;
		applyEnabled(computeEnabled());
	};

	if (pauseWhenUnfocused && typeof document !== 'undefined') {
		document.addEventListener('visibilitychange', handleVisibilityOrFocusChange);
	}
	if (pauseWhenUnfocused && typeof window !== 'undefined') {
		window.addEventListener('focus', handleVisibilityOrFocusChange);
		window.addEventListener('blur', handleVisibilityOrFocusChange);
	}
	handleVisibilityOrFocusChange();

	async function cleanup(_stopRemote: boolean = true): Promise<void> {
		if (cleanedUp) return;
		cleanedUp = true;
		if (pauseWhenUnfocused && typeof document !== 'undefined') {
			document.removeEventListener('visibilitychange', handleVisibilityOrFocusChange);
		}
		if (pauseWhenUnfocused && typeof window !== 'undefined') {
			window.removeEventListener('focus', handleVisibilityOrFocusChange);
			window.removeEventListener('blur', handleVisibilityOrFocusChange);
		}
		stopAllTracks(stream);
	}

	return {track, cleanup};
}

async function acquireChromiumDesktopPreviewStream(
	desktopCaptureSourceId: string,
	maxWidth: number | undefined,
	maxHeight: number | undefined,
	maxFrameRate: number,
): Promise<MediaStream> {
	const constraints: ChromiumDesktopCaptureConstraints = {
		mandatory: {
			chromeMediaSource: 'desktop',
			chromeMediaSourceId: desktopCaptureSourceId,
			...(maxWidth !== undefined ? {maxWidth} : {}),
			...(maxHeight !== undefined ? {maxHeight} : {}),
			maxFrameRate,
		},
	};
	return navigator.mediaDevices.getUserMedia({
		audio: false,
		video: constraints as unknown as MediaTrackConstraints,
	});
}

async function acquireCursorHiddenDisplayMediaPreviewStream(
	desktopCaptureSourceId: string,
	maxWidth: number | undefined,
	maxHeight: number | undefined,
	maxFrameRate: number,
): Promise<MediaStream> {
	const videoConstraints: DisplayMediaCursorPreviewConstraints = {
		cursor: 'never',
		...(maxWidth !== undefined ? {width: {ideal: maxWidth}} : {}),
		...(maxHeight !== undefined ? {height: {ideal: maxHeight}} : {}),
		frameRate: {ideal: maxFrameRate, max: maxFrameRate},
	};
	setDesktopSourceIntent({sourceId: desktopCaptureSourceId, includeAudio: false});
	try {
		return await navigator.mediaDevices.getDisplayMedia({
			audio: false,
			video: videoConstraints as MediaTrackConstraints,
		});
	} finally {
		clearDesktopSourceIntent();
	}
}

export async function createScreenChromiumPreviewBridge(
	desktopCaptureSourceId: string,
	options: ChromiumPreviewBridgeOptions = {},
): Promise<NativeScreenBridgeHandle> {
	const maxWidth = options.maxWidth;
	const maxHeight = options.maxHeight;
	const maxFrameRate = options.maxFrameRate ?? 60;
	const pauseWhenUnfocused = options.pauseWhenUnfocused ?? true;
	const previewPlatform = resolvePreviewPlatform(options);
	const useCursorHiddenPreview = shouldUseCursorHiddenDisplayMediaPreview(previewPlatform);
	const stream = useCursorHiddenPreview
		? await acquireCursorHiddenDisplayMediaPreviewStream(desktopCaptureSourceId, maxWidth, maxHeight, maxFrameRate)
		: await acquireChromiumDesktopPreviewStream(desktopCaptureSourceId, maxWidth, maxHeight, maxFrameRate);
	const handle = attachPreviewTrackLifecycle(stream, pauseWhenUnfocused);

	logger.debug('Chromium desktop preview track acquired', {
		desktopCaptureSourceId,
		maxWidth,
		maxHeight,
		maxFrameRate,
		cursorHiddenPreview: useCursorHiddenPreview,
	});

	return handle;
}
