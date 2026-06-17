// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import test from 'node:test';

interface FakeCommandLineState {
	switches: Map<string, string>;
}

function createFakeCommandLine(initial: Record<string, string | true> = {}) {
	const state: FakeCommandLineState = {switches: new Map()};
	for (const [name, value] of Object.entries(initial)) {
		state.switches.set(name, value === true ? '' : value);
	}
	return {
		state,
		appendSwitch(name: string, value?: string): void {
			state.switches.set(name, value ?? '');
		},
		getSwitchValue(name: string): string {
			return state.switches.get(name) ?? '';
		},
		hasSwitch(name: string): boolean {
			return state.switches.has(name);
		},
	};
}

const electronCommandLine = createFakeCommandLine();

test.mock.module('electron', {
	namedExports: {
		app: {commandLine: electronCommandLine},
		desktopCapturer: {getSources: async () => []},
		ipcMain: {handle: () => {}, on: () => {}},
		screen: {getAllDisplays: () => []},
		BrowserWindow: {getAllWindows: () => []},
	},
});

test.mock.module('electron-log', {
	defaultExport: {
		info: () => {},
		warn: () => {},
		error: () => {},
		debug: () => {},
	},
});

const {addLinuxScreenCapturePipeWireFeature, appendLinuxOzonePlatformHint, hasConfiguredOzonePlatformSwitch} =
	await import('./ChromiumRuntime');

const {resolveWaylandPortalDisplayMedia, DISPLAY_MEDIA_PORTAL_EMPTY_CHANNEL} = await import('./DisplayMedia');

function withPlatform<T>(platform: NodeJS.Platform, run: () => T): T {
	const original = Object.getOwnPropertyDescriptor(process, 'platform');
	Object.defineProperty(process, 'platform', {configurable: true, value: platform});
	try {
		return run();
	} finally {
		if (original) {
			Object.defineProperty(process, 'platform', original);
		}
	}
}

test('addLinuxScreenCapturePipeWireFeature enables WebRTCPipeWireCapturer only on Linux', () => {
	withPlatform('linux', () => {
		const features = new Set<string>();
		addLinuxScreenCapturePipeWireFeature(features);
		assert.ok(features.has('WebRTCPipeWireCapturer'));
	});
	withPlatform('darwin', () => {
		const features = new Set<string>();
		addLinuxScreenCapturePipeWireFeature(features);
		assert.equal(features.has('WebRTCPipeWireCapturer'), false);
	});
	withPlatform('win32', () => {
		const features = new Set<string>();
		addLinuxScreenCapturePipeWireFeature(features);
		assert.equal(features.has('WebRTCPipeWireCapturer'), false);
	});
});

test('appendLinuxOzonePlatformHint appends ozone-platform-hint=auto on Linux', () => {
	withPlatform('linux', () => {
		const commandLine = createFakeCommandLine();
		appendLinuxOzonePlatformHint(commandLine);
		assert.equal(commandLine.getSwitchValue('ozone-platform-hint'), 'auto');
		assert.ok(commandLine.hasSwitch('ozone-platform-hint'));
	});
});

test('appendLinuxOzonePlatformHint does not append on darwin or win32', () => {
	withPlatform('darwin', () => {
		const commandLine = createFakeCommandLine();
		appendLinuxOzonePlatformHint(commandLine);
		assert.equal(commandLine.hasSwitch('ozone-platform-hint'), false);
	});
	withPlatform('win32', () => {
		const commandLine = createFakeCommandLine();
		appendLinuxOzonePlatformHint(commandLine);
		assert.equal(commandLine.hasSwitch('ozone-platform-hint'), false);
	});
});

test('appendLinuxOzonePlatformHint does not override a user-supplied --ozone-platform', () => {
	withPlatform('linux', () => {
		const commandLine = createFakeCommandLine({'ozone-platform': 'x11'});
		assert.ok(hasConfiguredOzonePlatformSwitch(commandLine));
		appendLinuxOzonePlatformHint(commandLine);
		assert.equal(commandLine.hasSwitch('ozone-platform-hint'), false);
		assert.equal(commandLine.getSwitchValue('ozone-platform'), 'x11');
	});
});

test('appendLinuxOzonePlatformHint respects a user-supplied bare --ozone-platform switch', () => {
	withPlatform('linux', () => {
		const commandLine = createFakeCommandLine({'ozone-platform': true});
		assert.ok(hasConfiguredOzonePlatformSwitch(commandLine));
		appendLinuxOzonePlatformHint(commandLine);
		assert.equal(commandLine.hasSwitch('ozone-platform-hint'), false);
	});
});

test('resolveWaylandPortalDisplayMedia returns the first source when the portal provides one', async () => {
	const source = {id: 'screen:0:0', name: 'Screen 1'} as Electron.DesktopCapturerSource;
	const notifications: Array<{requestId: string; reason: string}> = [];
	const streams = await resolveWaylandPortalDisplayMedia({
		requestId: 'req-1',
		preference: 'monitor',
		getSources: async () => [source],
		notifyUnavailable: (requestId, reason) => notifications.push({requestId, reason}),
	});
	assert.deepEqual(streams, {video: source});
	assert.equal(notifications.length, 0);
});

test('resolveWaylandPortalDisplayMedia signals an empty portal result without a video stream', async () => {
	const notifications: Array<{requestId: string; reason: string}> = [];
	const streams = await resolveWaylandPortalDisplayMedia({
		requestId: 'req-2',
		preference: 'monitor',
		getSources: async () => [],
		notifyUnavailable: (requestId, reason) => notifications.push({requestId, reason}),
	});
	assert.equal(streams, null);
	assert.deepEqual(notifications, [{requestId: 'req-2', reason: 'empty'}]);
});

test('resolveWaylandPortalDisplayMedia signals a portal error when getSources rejects', async () => {
	const notifications: Array<{requestId: string; reason: string}> = [];
	const streams = await resolveWaylandPortalDisplayMedia({
		requestId: 'req-3',
		preference: 'window',
		getSources: async () => {
			throw new Error('xdg-desktop-portal unavailable');
		},
		notifyUnavailable: (requestId, reason) => notifications.push({requestId, reason}),
	});
	assert.equal(streams, null);
	assert.deepEqual(notifications, [{requestId: 'req-3', reason: 'error'}]);
});

test('the Wayland portal-unavailable IPC channel matches the wired renderer channel', () => {
	assert.equal(DISPLAY_MEDIA_PORTAL_EMPTY_CHANNEL, 'display-media-portal-empty');
});
