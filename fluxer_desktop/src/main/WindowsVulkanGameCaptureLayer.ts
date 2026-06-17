// SPDX-License-Identifier: AGPL-3.0-or-later

import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import log from 'electron-log';
import {
	enableWindowsGameCaptureModuleForCurrentProcess,
	WINDOWS_GAME_CAPTURE_MODULE_ENABLED,
} from './WindowsGameCapturePolicy';

const requireModule = createRequire(import.meta.url);
const VULKAN_IMPLICIT_LAYERS_REGISTRY_KEY = 'Software\\Khronos\\Vulkan\\ImplicitLayers';
const VULKAN_REGISTRY_ROOTS = ['HKCU', 'HKLM'] as const;

interface VulkanLayerRegistrationState {
	registered: boolean;
	manifestExists: boolean;
	dllExists: boolean;
	manifestPath: string | null;
}

type WindowsGameCaptureModule = {
	loadError?: Error | null;
	registerVulkanLayerManifest?: () => boolean;
	unregisterVulkanLayerManifest?: () => boolean;
	resolveVulkanLayerManifestPath?: () => string | null;
	getVulkanLayerRegistrationState?: () => VulkanLayerRegistrationState;
};

function parseRegistryValueNames(stdout: string): Array<string> {
	const valueNames: Array<string> = [];
	for (const line of stdout.split(/\r?\n/)) {
		const trimmed = line.trim();
		const match = trimmed.match(/^(.*?)\s+REG_DWORD\s+(?:0x[0-9a-f]+|\d+)$/i);
		if (!match) continue;
		const valueName = match[1].trim();
		if (valueName.length > 0) valueNames.push(valueName);
	}
	return valueNames;
}

function isFluxerGameCaptureVulkanLayerValue(valueName: string): boolean {
	const normalized = valueName.replace(/\//g, '\\').toLowerCase();
	if (!normalized.includes('\\@fluxer\\win-game-capture\\')) return false;
	return /\\fluxer-vulkan-layer\.win32-(?:x64|ia32|arm64)-msvc\.json$/.test(normalized);
}

function queryVulkanLayerRegistryValues(root: string): Array<string> {
	try {
		const stdout = execFileSync('reg.exe', ['query', `${root}\\${VULKAN_IMPLICIT_LAYERS_REGISTRY_KEY}`], {
			encoding: 'utf8',
			windowsHide: true,
		});
		return parseRegistryValueNames(stdout);
	} catch (error) {
		const status = (error as {status?: number} | null)?.status;
		if (status === 1) return [];
		throw error;
	}
}

function deleteVulkanLayerRegistryValue(root: string, valueName: string): void {
	execFileSync('reg.exe', ['delete', `${root}\\${VULKAN_IMPLICIT_LAYERS_REGISTRY_KEY}`, '/v', valueName, '/f'], {
		stdio: 'ignore',
		windowsHide: true,
	});
}

function removeStaleFluxerGameCaptureVulkanLayers(): void {
	if (process.platform !== 'win32') return;
	for (const root of VULKAN_REGISTRY_ROOTS) {
		const valueNames = queryVulkanLayerRegistryValues(root);
		for (const valueName of valueNames) {
			if (!isFluxerGameCaptureVulkanLayerValue(valueName)) continue;
			deleteVulkanLayerRegistryValue(root, valueName);
			log.info('[VulkanGameCaptureLayer] Removed stale Fluxer Vulkan layer registry value', {root, valueName});
		}
	}
}

function loadWindowsGameCaptureModule(): WindowsGameCaptureModule | null {
	if (process.platform !== 'win32') return null;
	if (!WINDOWS_GAME_CAPTURE_MODULE_ENABLED) return null;
	enableWindowsGameCaptureModuleForCurrentProcess();
	const addon = requireModule('@fluxer/win-game-capture') as WindowsGameCaptureModule;
	if (addon.loadError) {
		log.warn('[VulkanGameCaptureLayer] Native game capture addon unavailable', addon.loadError);
		return null;
	}
	return addon;
}

export function initializeWindowsVulkanGameCaptureLayer(): void {
	if (process.platform !== 'win32') return;
	try {
		if (!WINDOWS_GAME_CAPTURE_MODULE_ENABLED) {
			removeStaleFluxerGameCaptureVulkanLayers();
			log.info('[VulkanGameCaptureLayer] Native game capture disabled until Windows binaries are code signed');
			return;
		}
		const addon = loadWindowsGameCaptureModule();
		if (!addon) return;
		const registered = addon.registerVulkanLayerManifest?.() ?? false;
		const state = addon.getVulkanLayerRegistrationState?.() ?? null;
		log.info('[VulkanGameCaptureLayer] Vulkan implicit layer registration checked', {
			registered,
			manifestPath: addon.resolveVulkanLayerManifestPath?.() ?? null,
			state,
		});
	} catch (error) {
		log.warn('[VulkanGameCaptureLayer] Failed to register Vulkan implicit layer', error);
	}
}

export function unregisterWindowsVulkanGameCaptureLayer(): void {
	if (process.platform !== 'win32') return;
	try {
		if (!WINDOWS_GAME_CAPTURE_MODULE_ENABLED) {
			removeStaleFluxerGameCaptureVulkanLayers();
			return;
		}
		const addon = loadWindowsGameCaptureModule();
		if (!addon) return;
		const unregistered = addon.unregisterVulkanLayerManifest?.() ?? false;
		log.info('[VulkanGameCaptureLayer] Vulkan implicit layer unregistration attempted', {
			unregistered,
			manifestPath: addon.resolveVulkanLayerManifestPath?.() ?? null,
		});
	} catch (error) {
		log.warn('[VulkanGameCaptureLayer] Failed to unregister Vulkan implicit layer', error);
	}
}
