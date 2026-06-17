// SPDX-License-Identifier: AGPL-3.0-or-later

import {createRequire} from 'node:module';
import {WINDOWS_APP_USER_MODEL_ID} from '@electron/common/DesktopIdentity';
import {createChildLogger} from '@electron/common/Logger';
import {ipcMain} from 'electron';

const logger = createChildLogger('WindowsToast');
const requireModule = createRequire(import.meta.url);

interface ToastSupport {
	supported: boolean;
	reason?: string;
}

interface ToastBindingText {
	text: string;
	hintMaxLines?: number;
	hint?: 'attribution';
}

interface ToastImage {
	uri: string;
	placement?: 'hero' | 'appLogoOverride';
	hintCrop?: 'circle' | 'none';
	alt?: string;
}

interface ToastInputBox {
	id: string;
	type: 'text' | 'selection';
	placeholder?: string;
	title?: string;
	options?: ReadonlyArray<{id: string; content: string}>;
}

interface ToastAction {
	label: string;
	args: string;
	activationType?: 'foreground' | 'background' | 'protocol';
	imageUri?: string;
	hintInputId?: string;
}

interface WindowsToastNotifyRequest {
	tag?: string;
	group?: string;
	expirationTime?: string;
	scenario?: 'default' | 'urgent' | 'reminder' | 'incomingCall' | 'alarm';
	audio?: 'default' | 'silent' | {silent?: boolean; loop?: boolean; src?: string};
	lines: ReadonlyArray<ToastBindingText>;
	images?: ReadonlyArray<ToastImage>;
	inputs?: ReadonlyArray<ToastInputBox>;
	actions?: ReadonlyArray<ToastAction>;
}

interface WinToastModule {
	isSupported: () => ToastSupport;
	notify: (opts: WindowsToastNotifyRequest & {aumid: string}) => Promise<void>;
	dismiss: (opts: {aumid: string; tag: string; group?: string}) => Promise<void>;
	clear: (opts: {aumid: string}) => Promise<void>;
	loadError: Error | null;
}

let cached: WinToastModule | null | undefined;
let lastSupport: ToastSupport | undefined;

function loadAddon(): WinToastModule | null {
	if (cached !== undefined) return cached;
	if (process.platform !== 'win32') {
		cached = null;
		return cached;
	}
	try {
		const mod = requireModule('@fluxer/win-toast') as WinToastModule;
		if (mod.loadError) {
			logger.warn('@fluxer/win-toast loaded but reported load error', {error: mod.loadError});
			cached = null;
			return cached;
		}
		cached = mod;
		return cached;
	} catch (error) {
		logger.info('@fluxer/win-toast not available; rich toasts disabled', {error});
		cached = null;
		return cached;
	}
}

function aumid(): string | null {
	return WINDOWS_APP_USER_MODEL_ID || null;
}

function isWindowsToastSupported(): ToastSupport {
	if (lastSupport) return lastSupport;
	if (process.platform !== 'win32') {
		lastSupport = {supported: false, reason: 'not-windows'};
		return lastSupport;
	}
	const mod = loadAddon();
	if (!mod) {
		lastSupport = {supported: false, reason: 'addon-not-installed'};
		return lastSupport;
	}
	if (aumid() == null) {
		lastSupport = {supported: false, reason: 'no-aumid'};
		return lastSupport;
	}
	try {
		lastSupport = mod.isSupported();
	} catch (error) {
		lastSupport = {
			supported: false,
			reason: `isSupported threw: ${error instanceof Error ? error.message : String(error)}`,
		};
	}
	return lastSupport;
}

async function showWindowsToast(req: WindowsToastNotifyRequest): Promise<boolean> {
	const support = isWindowsToastSupported();
	if (!support.supported) return false;
	const mod = loadAddon();
	const id = aumid();
	if (!mod || id == null) return false;
	try {
		await mod.notify({...req, aumid: id});
		return true;
	} catch (error) {
		logger.warn('win-toast notify failed', {error});
		return false;
	}
}

async function dismissWindowsToast(tag: string, group?: string): Promise<void> {
	const mod = loadAddon();
	const id = aumid();
	if (!mod || id == null) return;
	try {
		await mod.dismiss({aumid: id, tag, group});
	} catch (error) {
		logger.debug('win-toast dismiss failed (often benign)', {error});
	}
}

async function clearWindowsToasts(): Promise<void> {
	const mod = loadAddon();
	const id = aumid();
	if (!mod || id == null) return;
	try {
		await mod.clear({aumid: id});
	} catch (error) {
		logger.debug('win-toast clear failed', {error});
	}
}

let registered = false;

export function registerWindowsToastIpcHandlers(): void {
	if (registered) return;
	registered = true;
	ipcMain.handle('win-toast:is-supported', (): ToastSupport => isWindowsToastSupported());
	ipcMain.handle(
		'win-toast:notify',
		async (_event, req: WindowsToastNotifyRequest): Promise<boolean> => showWindowsToast(req),
	);
	ipcMain.handle(
		'win-toast:dismiss',
		async (_event, opts: {tag: string; group?: string}): Promise<void> => dismissWindowsToast(opts.tag, opts.group),
	);
	ipcMain.handle('win-toast:clear', async (): Promise<void> => clearWindowsToasts());
}

function _cleanupWindowsToastIpcHandlers(): void {
	if (!registered) return;
	ipcMain.removeHandler('win-toast:is-supported');
	ipcMain.removeHandler('win-toast:notify');
	ipcMain.removeHandler('win-toast:dismiss');
	ipcMain.removeHandler('win-toast:clear');
	registered = false;
}
