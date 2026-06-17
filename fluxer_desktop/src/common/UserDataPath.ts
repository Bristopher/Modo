// SPDX-License-Identifier: AGPL-3.0-or-later

import fs from 'node:fs';
import path from 'node:path';
import {BUILD_CHANNEL, type BuildChannel} from '@electron/common/BuildChannel';
import {app} from 'electron';

interface UserDataPaths {
	readonly channel: BuildChannel;
	readonly directoryName: string;
	readonly base: string;
	readonly portable: boolean;
}

interface ChannelStorageDirectoryMap {
	stable: string;
	canary: string;
}

const channelStorageDirectoryMap: ChannelStorageDirectoryMap = {
	stable: 'fluxer',
	canary: 'fluxercanary',
};

let portableMode = false;

function getPortableBasePath(): string {
	let appDir: string;
	if (process.platform === 'darwin') {
		const match = process.execPath.match(/^(.+?)\.app\//);
		appDir = match ? path.dirname(`${match[1]}.app`) : path.dirname(process.execPath);
	} else if (process.env.APPIMAGE) {
		appDir = path.dirname(process.env.APPIMAGE);
	} else {
		appDir = path.dirname(process.execPath);
	}
	return path.join(appDir, 'data');
}

function detectPortableMode(): boolean {
	if (process.argv.includes('--fluxer-portable')) return true;
	const envValue = process.env.FLUXER_PORTABLE;
	if (envValue && ['1', 'true', 'yes', 'on'].includes(envValue.trim().toLowerCase())) return true;
	const markerLocations = [path.join(path.dirname(process.execPath), '.portable')];
	if (process.platform === 'darwin') {
		const match = process.execPath.match(/^(.+?)\.app\//);
		if (match) {
			markerLocations.push(path.join(path.dirname(`${match[1]}.app`), '.portable'));
		}
	}
	if (process.env.APPIMAGE) {
		markerLocations.push(path.join(path.dirname(process.env.APPIMAGE), '.portable'));
	}
	return markerLocations.some((location) => {
		try {
			return fs.existsSync(location);
		} catch {
			return false;
		}
	});
}

function resolveUserDataPaths(channel: BuildChannel): {
	directoryName: string;
	base: string;
	portable: boolean;
} {
	const directoryName = channelStorageDirectoryMap[channel];
	const portable = detectPortableMode();
	portableMode = portable;
	if (portable) {
		const base = path.join(getPortableBasePath(), directoryName);
		fs.mkdirSync(base, {recursive: true});
		return {directoryName, base, portable};
	}
	const appDataPath = app.getPath('appData');
	const base = path.join(appDataPath, directoryName);
	return {directoryName, base, portable};
}

export function isPortableMode(): boolean {
	return portableMode;
}

export function configureUserDataPath(): UserDataPaths {
	const channel = BUILD_CHANNEL;
	const {directoryName, base, portable} = resolveUserDataPaths(channel);
	app.setPath('userData', base);
	if (portable) {
		try {
			app.setPath('sessionData', path.join(base, 'session'));
			fs.mkdirSync(path.join(base, 'session'), {recursive: true});
		} catch {}
		try {
			const logsPath = path.join(base, 'logs');
			app.setPath('logs', logsPath);
			fs.mkdirSync(logsPath, {recursive: true});
		} catch {}
		try {
			const crashDumpsPath = path.join(base, 'crash-dumps');
			app.setPath('crashDumps', crashDumpsPath);
			fs.mkdirSync(crashDumpsPath, {recursive: true});
		} catch {}
	}
	return {
		channel,
		directoryName,
		base,
		portable,
	};
}
