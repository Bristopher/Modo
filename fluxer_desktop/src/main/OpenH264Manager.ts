// SPDX-License-Identifier: AGPL-3.0-or-later

import {execFile} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import type {OpenH264Status} from '@electron/common/Types';
import {downloadFile} from '@electron/main/FileDownloads';
import {app} from 'electron';
import log from 'electron-log';

const OPENH264_VERSION = '2.5.1';
const CISCO_CDN_BASE = 'https://ciscobinary.openh264.org';
const PREFERENCES_FILE = 'preferences.json';
const STATUS_FILE = 'status.json';
const DECOMPRESS_TIMEOUT_MS = 30_000;

interface PersistedPreferences {
	enabled?: boolean;
}

interface PersistedStatus {
	version: string;
	binaryPath: string;
	downloadedAt: string;
}

function getOpenH264Dir(): string {
	return path.join(app.getPath('userData'), 'openh264');
}

function getStatusFilePath(): string {
	return path.join(getOpenH264Dir(), STATUS_FILE);
}

function getPreferencesFilePath(): string {
	return path.join(getOpenH264Dir(), PREFERENCES_FILE);
}

function getBinaryFilename(): string | null {
	if (process.platform !== 'linux') return null;
	return `libopenh264-${OPENH264_VERSION}-linux64.7.so`;
}

function getDownloadUrl(): string | null {
	const filename = getBinaryFilename();
	if (!filename) return null;
	return `${CISCO_CDN_BASE}/${filename}.bz2`;
}

function _getBinaryPath(): string | null {
	const filename = getBinaryFilename();
	if (!filename) return null;
	return path.join(getOpenH264Dir(), filename);
}

function readPersistedStatus(): PersistedStatus | null {
	try {
		const data = JSON.parse(fs.readFileSync(getStatusFilePath(), 'utf8')) as Partial<PersistedStatus>;
		if (typeof data.version === 'string' && typeof data.binaryPath === 'string') {
			return data as PersistedStatus;
		}
		return null;
	} catch {
		return null;
	}
}

function readPersistedEnabled(): boolean {
	try {
		const data = JSON.parse(fs.readFileSync(getPreferencesFilePath(), 'utf8')) as PersistedPreferences;
		return typeof data.enabled === 'boolean' ? data.enabled : true;
	} catch {
		return true;
	}
}

function writePersistedEnabled(value: boolean): void {
	try {
		fs.mkdirSync(getOpenH264Dir(), {recursive: true});
		fs.writeFileSync(getPreferencesFilePath(), JSON.stringify({enabled: value}, null, 2), 'utf8');
	} catch (error) {
		log.warn('[OpenH264] Failed to write preferences file', {error});
	}
}

function writePersistedStatus(status: PersistedStatus): void {
	try {
		fs.mkdirSync(getOpenH264Dir(), {recursive: true});
		fs.writeFileSync(getStatusFilePath(), JSON.stringify(status, null, 2), 'utf8');
	} catch (error) {
		log.warn('[OpenH264] Failed to write status file', {error});
	}
}

function binaryExists(): boolean {
	const persisted = readPersistedStatus();
	if (!persisted) return false;
	if (persisted.version !== OPENH264_VERSION) return false;
	try {
		fs.accessSync(persisted.binaryPath, fs.constants.R_OK);
		return true;
	} catch {
		return false;
	}
}

function getBinaryPathIfReady(): string | null {
	const persisted = readPersistedStatus();
	if (!persisted) return null;
	if (persisted.version !== OPENH264_VERSION) return null;
	try {
		fs.accessSync(persisted.binaryPath, fs.constants.R_OK);
		return persisted.binaryPath;
	} catch {
		return null;
	}
}

function decompressBz2(compressedPath: string): Promise<void> {
	return new Promise((resolve, reject) => {
		execFile('bunzip2', ['--keep', '--force', compressedPath], {timeout: DECOMPRESS_TIMEOUT_MS}, (error) => {
			if (error) {
				reject(new Error(`bunzip2 failed: ${error.message}`));
				return;
			}
			resolve();
		});
	});
}

let enabled: boolean | null = null;
let downloading = false;
let lastError: string | null = null;

function getEnabled(): boolean {
	if (enabled === null) {
		enabled = readPersistedEnabled();
	}
	return enabled;
}

async function downloadOpenH264(): Promise<void> {
	const url = getDownloadUrl();
	const filename = getBinaryFilename();
	if (!url || !filename) {
		lastError = 'Unsupported platform';
		return;
	}

	const dir = getOpenH264Dir();
	const compressedPath = path.join(dir, `${filename}.bz2`);
	const binaryPath = path.join(dir, filename);

	downloading = true;
	lastError = null;

	try {
		fs.mkdirSync(dir, {recursive: true});

		log.info('[OpenH264] Downloading from Cisco CDN', {url, version: OPENH264_VERSION});
		await downloadFile(url, compressedPath);

		log.info('[OpenH264] Decompressing', {compressedPath});
		await decompressBz2(compressedPath);

		if (fs.existsSync(binaryPath)) {
			writePersistedStatus({
				version: OPENH264_VERSION,
				binaryPath,
				downloadedAt: new Date().toISOString(),
			});
			log.info('[OpenH264] Binary ready', {binaryPath, version: OPENH264_VERSION});
		} else {
			lastError = 'Decompression produced no output file';
			log.warn('[OpenH264] Binary not found after decompression', {binaryPath});
		}

		try {
			fs.unlinkSync(compressedPath);
		} catch {}
	} catch (error) {
		lastError = error instanceof Error ? error.message : String(error);
		log.warn('[OpenH264] Download failed', {error: lastError});
	} finally {
		downloading = false;
	}
}

export function getStatus(): OpenH264Status {
	if (process.platform !== 'linux') {
		return {
			enabled: false,
			downloaded: false,
			downloading: false,
			version: null,
			error: null,
		};
	}
	const binaryPath = getBinaryPathIfReady();
	return {
		enabled: getEnabled(),
		downloaded: binaryPath !== null,
		downloading,
		version: binaryPath !== null ? OPENH264_VERSION : null,
		error: lastError,
	};
}

export async function setEnabled(value: boolean): Promise<OpenH264Status> {
	if (process.platform !== 'linux') {
		enabled = false;
		return getStatus();
	}
	enabled = value;
	writePersistedEnabled(value);
	if (getEnabled() && !binaryExists() && !downloading) {
		void downloadOpenH264();
	}
	return getStatus();
}

export function initOpenH264(): void {
	if (process.platform !== 'linux') return;
	if (getEnabled() && !binaryExists()) {
		void downloadOpenH264();
	}
}

export function appendOpenH264Switches(): void {
	if (process.platform !== 'linux' || !getEnabled()) return;
	const binaryPath = getBinaryPathIfReady();
	if (!binaryPath) return;
	app.commandLine.appendSwitch('enable-libopenh264');
	app.commandLine.appendSwitch('openh264-library-path', binaryPath);
	log.info('[OpenH264] Chromium OpenH264 library path configured', {binaryPath, version: OPENH264_VERSION});
}
