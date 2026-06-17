// SPDX-License-Identifier: AGPL-3.0-or-later

import fs from 'node:fs';
import path from 'node:path';
import {DESKTOP_APP_NAME} from '@electron/common/DesktopIdentity';
import {createChildLogger} from '@electron/common/Logger';
import {app} from 'electron';

const logger = createChildLogger('MainI18n');
const FALLBACK_STRINGS: Readonly<Record<string, string>> = Object.freeze({
	'desktop.appMenu.about': 'About {appName}',
	'desktop.appMenu.preferences': 'Preferences...',
	'desktop.appMenu.preferencesPlain': 'Preferences',
	'desktop.appMenu.hide': 'Hide {appName}',
	'desktop.appMenu.quit': 'Quit {appName}',
	'desktop.appMenu.file': 'File',
	'desktop.appMenu.edit': 'Edit',
	'desktop.appMenu.speech': 'Speech',
	'desktop.appMenu.view': 'View',
	'desktop.appMenu.toggleDeveloperTools': 'Toggle Developer Tools',
	'desktop.appMenu.actualSize': 'Actual Size',
	'desktop.appMenu.zoomIn': 'Zoom In',
	'desktop.appMenu.zoomOut': 'Zoom Out',
	'desktop.appMenu.window': 'Window',
	'desktop.appMenu.close': 'Close',
	'desktop.appMenu.help': 'Help',
	'desktop.appMenu.website': 'Website',
	'desktop.appMenu.github': 'GitHub',
	'desktop.appMenu.reportIssue': 'Report Issue',
	'desktop.appMenu.troubleshooting': 'Troubleshooting',
	'desktop.troubleshooting.disableHardwareAccelerationAndRestart': 'Disable Hardware Acceleration and Restart',
	'desktop.troubleshooting.enableHardwareAccelerationAndRestart': 'Enable Hardware Acceleration and Restart',
	'desktop.troubleshooting.reload': 'Reload',
	'desktop.troubleshooting.resetAppDataAndRestart': 'Reset App Data and Restart',
	'desktop.troubleshooting.resetTitle': 'Reset App Data',
	'desktop.troubleshooting.resetMessage': 'Are you sure you want to reset Fluxer\u2019s app data?',
	'desktop.troubleshooting.resetDetail':
		'This will sign you out, clear cached files and stored sessions, and restart the desktop app. Your account, communities, and messages on the server are not affected.',
	'desktop.troubleshooting.resetConfirm': 'Reset and Restart',
	'desktop.troubleshooting.resetCancel': 'Cancel',
	'desktop.tray.show': 'Show {appName}',
	'desktop.tray.hide': 'Hide {appName}',
	'desktop.tray.openSettings': 'Open Settings',
	'desktop.tray.status': 'Status',
	'desktop.tray.statusOnline': 'Online',
	'desktop.tray.statusIdle': 'Idle',
	'desktop.tray.statusDnd': 'Do Not Disturb',
	'desktop.tray.statusInvisible': 'Invisible',
	'desktop.tray.muteMic': 'Mute Microphone',
	'desktop.tray.unmuteMic': 'Unmute Microphone',
	'desktop.tray.deafen': 'Deafen',
	'desktop.tray.undeafen': 'Undeafen',
	'desktop.tray.disconnectFrom': 'Disconnect from {channel}',
	'desktop.tray.disconnectVoice': 'Disconnect from Voice',
	'desktop.tray.checkForUpdates': 'Check for Updates',
	'desktop.tray.copyBuildInfo': 'Copy Build Info',
	'desktop.tray.restart': 'Restart {appName}',
	'desktop.tray.quit': 'Quit {appName}',
	'desktop.autostart.portalReason': 'Start {appName} automatically when you sign in.',
	'desktop.jumpList.tasks': 'Tasks',
	'desktop.jumpList.openSettings': 'Open Settings',
	'desktop.jumpList.openSettingsDescription': 'Open {appName} settings',
	'desktop.jumpList.newDirectMessage': 'New Direct Message',
	'desktop.jumpList.newDirectMessageDescription': 'Compose a new direct message',
	'desktop.jumpList.recent': 'Recent',
});
const STORAGE_FILE_NAME = 'native-strings.json';

interface PersistedStrings {
	locale: string;
	strings: Record<string, string>;
}

let currentLocale = 'en-US';
let currentStrings: Record<string, string> = {...FALLBACK_STRINGS};

const subscribers = new Set<() => void>();

function getStorageFilePath(): string {
	return path.join(app.getPath('userData'), STORAGE_FILE_NAME);
}

function readPersisted(): PersistedStrings | null {
	try {
		const raw = fs.readFileSync(getStorageFilePath(), 'utf8');
		const parsed = JSON.parse(raw) as PersistedStrings;
		if (typeof parsed?.locale === 'string' && parsed.strings && typeof parsed.strings === 'object') {
			return parsed;
		}
	} catch {}
	return null;
}

function writePersisted(payload: PersistedStrings): void {
	try {
		fs.writeFileSync(getStorageFilePath(), JSON.stringify(payload), {encoding: 'utf8', mode: 0o600});
	} catch (error) {
		logger.warn('Failed to persist native strings', {error});
	}
}

export function initializeMainI18n(): void {
	const persisted = readPersisted();
	if (!persisted) return;
	currentLocale = persisted.locale;
	currentStrings = {...FALLBACK_STRINGS, ...persisted.strings};
	logger.info('Loaded persisted native strings', {locale: currentLocale, count: Object.keys(persisted.strings).length});
}

export function setNativeStrings(locale: string, strings: Record<string, string>): void {
	const next = {...FALLBACK_STRINGS, ...strings};
	if (locale === currentLocale && shallowEqualStrings(currentStrings, next)) {
		return;
	}
	currentLocale = locale;
	currentStrings = next;
	writePersisted({locale, strings});
	for (const subscriber of subscribers) {
		try {
			subscriber();
		} catch (error) {
			logger.warn('Locale subscriber threw', {error});
		}
	}
}

function shallowEqualStrings(a: Record<string, string>, b: Record<string, string>): boolean {
	const aKeys = Object.keys(a);
	if (aKeys.length !== Object.keys(b).length) return false;
	for (const key of aKeys) {
		if (a[key] !== b[key]) return false;
	}
	return true;
}

export function getNativeLocale(): string {
	return currentLocale;
}

export function t(key: string, vars?: Record<string, string>): string {
	const template = currentStrings[key] ?? FALLBACK_STRINGS[key] ?? key;
	if (!template.includes('{')) return template;
	const resolved: Record<string, string> = {appName: DESKTOP_APP_NAME, ...(vars ?? {})};
	return template.replace(/\{(\w+)\}/g, (match, name: string) => (name in resolved ? resolved[name] : match));
}

export function onLocaleChange(callback: () => void): () => void {
	subscribers.add(callback);
	return () => subscribers.delete(callback);
}
