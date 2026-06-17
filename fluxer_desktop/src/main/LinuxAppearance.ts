// SPDX-License-Identifier: AGPL-3.0-or-later

import {createRequire} from 'node:module';
import {createChildLogger} from '@electron/common/Logger';
import type {LinuxAppearanceSnapshot} from '@electron/common/Types';
import {getLinuxPortalsMode} from '@electron/main/LaunchOptions';

export type {LinuxAppearanceSnapshot} from '@electron/common/Types';

const logger = createChildLogger('LinuxAppearance');
const requireModule = createRequire(import.meta.url);

interface SettingsChangeEvent {
	namespace: string;
	key: string;
	uint32?: number;
	accent?: {r: number; g: number; b: number};
}

interface SettingsCtor {
	new (onChange: (event: SettingsChangeEvent) => void): {close: () => void};
}

interface PortalsModule {
	readColorScheme: () => 'no-preference' | 'prefer-dark' | 'prefer-light';
	readContrast: () => 'no-preference' | 'high';
	readAccentColor: () => {r: number; g: number; b: number} | null;
	Settings: SettingsCtor | null;
	loadError: Error | null;
}

let cached: PortalsModule | null | undefined;

function loadAddon(): PortalsModule | null {
	if (cached !== undefined) return cached;
	if (process.platform !== 'linux') {
		cached = null;
		return cached;
	}
	if (getLinuxPortalsMode(process.argv) === 'off') {
		logger.info('@fluxer/linux-portals disabled by launch diagnostics; appearance reads disabled');
		cached = null;
		return cached;
	}
	try {
		const mod = requireModule('@fluxer/linux-portals') as PortalsModule;
		if (mod.loadError) {
			logger.info('@fluxer/linux-portals reported load error', {error: mod.loadError});
			cached = null;
			return cached;
		}
		cached = mod;
		return cached;
	} catch (error) {
		logger.info('@fluxer/linux-portals not available; appearance reads disabled', {error});
		cached = null;
		return cached;
	}
}

export function readLinuxAppearance(): LinuxAppearanceSnapshot {
	const mod = loadAddon();
	if (!mod) {
		return {colorScheme: 'no-preference', contrast: 'no-preference', accent: null};
	}
	return {
		colorScheme: mod.readColorScheme(),
		contrast: mod.readContrast(),
		accent: mod.readAccentColor(),
	};
}

type LinuxAppearanceListener = (snapshot: LinuxAppearanceSnapshot) => void;

export interface LinuxAppearanceSubscription {
	close(): void;
}

export function subscribeLinuxAppearance(listener: LinuxAppearanceListener): LinuxAppearanceSubscription | null {
	const mod = loadAddon();
	if (!mod?.Settings) return null;
	let closed = false;
	let instance: {close: () => void} | null = null;
	try {
		instance = new mod.Settings((event) => {
			if (closed) return;
			const snapshot: LinuxAppearanceSnapshot = {
				colorScheme: mod.readColorScheme(),
				contrast: mod.readContrast(),
				accent: mod.readAccentColor(),
			};
			if (event.namespace === 'org.freedesktop.appearance') {
				if (event.key === 'color-scheme' && event.uint32 != null) {
					snapshot.colorScheme =
						event.uint32 === 1 ? 'prefer-dark' : event.uint32 === 2 ? 'prefer-light' : 'no-preference';
				} else if (event.key === 'contrast' && event.uint32 != null) {
					snapshot.contrast = event.uint32 === 1 ? 'high' : 'no-preference';
				} else if (event.key === 'accent-color') {
					snapshot.accent =
						event.accent && event.accent.r >= 0 && event.accent.g >= 0 && event.accent.b >= 0 ? event.accent : null;
				}
			}
			listener(snapshot);
		});
	} catch (error) {
		logger.warn('linux-portals Settings subscribe failed', {error});
		return null;
	}
	return {
		close: () => {
			if (closed) return;
			closed = true;
			try {
				instance?.close();
			} catch (error) {
				logger.warn('linux-portals Settings close threw', {error});
			}
		},
	};
}
