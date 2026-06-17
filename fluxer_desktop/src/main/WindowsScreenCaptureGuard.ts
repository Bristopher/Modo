// SPDX-License-Identifier: AGPL-3.0-or-later

import {BrowserWindow, screen} from 'electron';
import log from 'electron-log';

const PENDING_GUARD_TTL_MS = 120000;
const GUARD_SIZE_DIP = 2;

interface ActiveWindowsScreenCaptureGuard {
	window: BrowserWindow;
	sourceId: string;
	displayId?: string;
	retained: boolean;
	timeout: NodeJS.Timeout | null;
}

let activeGuard: ActiveWindowsScreenCaptureGuard | null = null;
let guardRetentionRequested = false;

function isScreenSource(source: Electron.DesktopCapturerSource): boolean {
	return source.id.startsWith('screen:');
}

function parseScreenSourceOrdinal(sourceId: string): number | null {
	const match = /^screen:([0-9]+):(?:0|1)$/.exec(sourceId);
	if (!match) return null;
	const parsed = Number.parseInt(match[1], 10);
	return Number.isSafeInteger(parsed) && parsed >= 0 ? parsed : null;
}

function resolveDisplayForSource(source: Electron.DesktopCapturerSource): Electron.Display | null {
	const displays = screen.getAllDisplays();
	if (source.display_id) {
		const byDisplayId = displays.find((display) => String(display.id) === String(source.display_id));
		if (byDisplayId) return byDisplayId;
	}
	const ordinal = parseScreenSourceOrdinal(source.id);
	if (ordinal != null && displays[ordinal]) {
		return displays[ordinal];
	}
	return screen.getPrimaryDisplay();
}

function clearGuardTimeout(guard: ActiveWindowsScreenCaptureGuard): void {
	if (guard.timeout) {
		clearTimeout(guard.timeout);
		guard.timeout = null;
	}
}

function armPendingGuardTimeout(guard: ActiveWindowsScreenCaptureGuard): void {
	clearGuardTimeout(guard);
	guard.timeout = setTimeout(() => {
		if (activeGuard !== guard || guard.retained) return;
		destroyActiveGuard('pending-timeout');
	}, PENDING_GUARD_TTL_MS);
}

function retainGuardIfRequested(guard: ActiveWindowsScreenCaptureGuard): void {
	guard.retained = guardRetentionRequested;
	if (guard.retained) {
		clearGuardTimeout(guard);
	} else {
		armPendingGuardTimeout(guard);
	}
}

function createGuardWindow(display: Electron.Display): BrowserWindow {
	const {x, y, width, height} = display.bounds;
	const guardWindow = new BrowserWindow({
		x: x + Math.max(0, width - GUARD_SIZE_DIP),
		y: y + Math.max(0, height - GUARD_SIZE_DIP),
		width: GUARD_SIZE_DIP,
		height: GUARD_SIZE_DIP,
		frame: false,
		resizable: false,
		movable: false,
		minimizable: false,
		maximizable: false,
		closable: false,
		focusable: false,
		skipTaskbar: true,
		transparent: true,
		hasShadow: false,
		show: false,
		backgroundColor: '#00000000',
		webPreferences: {
			contextIsolation: true,
			nodeIntegration: false,
			sandbox: true,
		},
	});
	guardWindow.setIgnoreMouseEvents(true, {forward: true});
	try {
		guardWindow.setContentProtection(true);
	} catch (error) {
		log.debug('[WindowsScreenCaptureGuard] Failed to set content protection', {error});
	}
	try {
		guardWindow.setAlwaysOnTop(true, 'screen-saver', 1);
	} catch {
		guardWindow.setAlwaysOnTop(true);
	}
	try {
		guardWindow.setVisibleOnAllWorkspaces(true, {visibleOnFullScreen: true});
	} catch {}
	void guardWindow
		.loadURL(
			'data:text/html;charset=utf-8,' +
				encodeURIComponent(
					'<!doctype html><html><head><style>html,body{margin:0;width:100%;height:100%;overflow:hidden;background:rgba(0,0,0,0.01);}</style></head><body></body></html>',
				),
		)
		.catch((error) => {
			log.debug('[WindowsScreenCaptureGuard] Failed to load guard surface', {error});
		});
	guardWindow.showInactive();
	return guardWindow;
}

export function startWindowsScreenCaptureGuardForSource(source: Electron.DesktopCapturerSource): void {
	if (process.platform !== 'win32') return;
	if (!isScreenSource(source)) return;
	if (activeGuard?.sourceId === source.id && !activeGuard.window.isDestroyed()) {
		retainGuardIfRequested(activeGuard);
		return;
	}
	destroyActiveGuard('replace');
	const display = resolveDisplayForSource(source);
	if (!display) return;
	try {
		const guardWindow = createGuardWindow(display);
		const guard: ActiveWindowsScreenCaptureGuard = {
			window: guardWindow,
			sourceId: source.id,
			displayId: source.display_id,
			retained: guardRetentionRequested,
			timeout: null,
		};
		guardWindow.once('closed', () => {
			if (activeGuard === guard) {
				clearGuardTimeout(guard);
				activeGuard = null;
			}
		});
		activeGuard = guard;
		retainGuardIfRequested(guard);
		log.info('[WindowsScreenCaptureGuard] Armed monitor composition guard', {
			sourceId: source.id,
			displayId: source.display_id,
			retained: guard.retained,
			bounds: display.bounds,
		});
	} catch (error) {
		log.warn('[WindowsScreenCaptureGuard] Failed to arm monitor composition guard', {sourceId: source.id, error});
	}
}

export function retainWindowsScreenCaptureGuard(): void {
	if (process.platform !== 'win32') return;
	guardRetentionRequested = true;
	if (!activeGuard) return;
	activeGuard.retained = true;
	clearGuardTimeout(activeGuard);
}

function destroyActiveGuard(reason: string): void {
	const guard = activeGuard;
	if (!guard) return;
	activeGuard = null;
	clearGuardTimeout(guard);
	if (!guard.window.isDestroyed()) {
		try {
			guard.window.destroy();
		} catch (error) {
			log.debug('[WindowsScreenCaptureGuard] Failed to destroy guard window', {reason, error});
		}
	}
	log.info('[WindowsScreenCaptureGuard] Stopped monitor composition guard', {
		reason,
		sourceId: guard.sourceId,
		displayId: guard.displayId,
	});
}

export function stopWindowsScreenCaptureGuard(reason: string): void {
	guardRetentionRequested = false;
	destroyActiveGuard(reason);
}
