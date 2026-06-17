// SPDX-License-Identifier: AGPL-3.0-or-later

import {createRequire} from 'node:module';
import {createChildLogger} from '@electron/common/Logger';
import {ipcMain, systemPreferences} from 'electron';

const logger = createChildLogger('MacTcc');
const requireModule = createRequire(import.meta.url);

type TccStatus = 'granted' | 'denied' | 'not-determined';
type TccSurface = 'screen-recording' | 'input-monitoring';

interface MacTccModule {
	screenRecordingStatus: () => TccStatus;
	requestScreenRecording: () => TccStatus;
	inputMonitoringStatus: () => TccStatus;
	requestInputMonitoring: () => TccStatus;
	loadError: Error | null;
}

let cached: MacTccModule | null | undefined;

function loadAddon(): MacTccModule | null {
	if (cached !== undefined) return cached;
	if (process.platform !== 'darwin') {
		cached = null;
		return cached;
	}
	try {
		const mod = requireModule('@fluxer/mac-tcc') as MacTccModule;
		if (mod.loadError) {
			logger.info('@fluxer/mac-tcc reported load error', {error: mod.loadError});
			cached = null;
			return cached;
		}
		cached = mod;
		return cached;
	} catch (error) {
		logger.info('@fluxer/mac-tcc not available; TCC pre-flight disabled', {error});
		cached = null;
		return cached;
	}
}

function screenRecordingStatusWithoutAddon(): TccStatus {
	switch (systemPreferences.getMediaAccessStatus('screen')) {
		case 'granted':
			return 'granted';
		case 'denied':
		case 'restricted':
			return 'denied';
		default:
			return 'not-determined';
	}
}

function statusWithoutAddon(surface: TccSurface): TccStatus {
	if (process.platform !== 'darwin') return 'not-determined';
	switch (surface) {
		case 'screen-recording':
			return screenRecordingStatusWithoutAddon();
		case 'input-monitoring':
			return 'not-determined';
	}
}

function statusOf(surface: TccSurface): TccStatus {
	const mod = loadAddon();
	if (!mod) return statusWithoutAddon(surface);
	switch (surface) {
		case 'screen-recording':
			return mod.screenRecordingStatus();
		case 'input-monitoring':
			return mod.inputMonitoringStatus();
	}
}

function requestOf(surface: TccSurface): TccStatus {
	const mod = loadAddon();
	if (!mod) return statusWithoutAddon(surface);
	switch (surface) {
		case 'screen-recording':
			return mod.requestScreenRecording();
		case 'input-monitoring':
			return mod.requestInputMonitoring();
	}
}

export function getTccStatus(surface: TccSurface): TccStatus {
	return statusOf(surface);
}

function _requestTcc(surface: TccSurface): TccStatus {
	return requestOf(surface);
}

let registered = false;

export function registerMacTccIpcHandlers(): void {
	if (registered) return;
	registered = true;
	ipcMain.handle('mac-tcc:status', (_event, surface: TccSurface): TccStatus => statusOf(surface));
	ipcMain.handle('mac-tcc:request', (_event, surface: TccSurface): TccStatus => requestOf(surface));
}

function _cleanupMacTccIpcHandlers(): void {
	if (!registered) return;
	ipcMain.removeHandler('mac-tcc:status');
	ipcMain.removeHandler('mac-tcc:request');
	registered = false;
}
