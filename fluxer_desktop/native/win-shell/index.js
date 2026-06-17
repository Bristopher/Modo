// SPDX-License-Identifier: AGPL-3.0-or-later

const {existsSync} = require('node:fs');
const {join, sep} = require('node:path');
const {createNativeLoadError, loadNativeBinding} = require('./loader-diagnostics.cjs');
const MODULE_NAME = '@fluxer/win-shell';

function resolveNativeRoot() {
	const asarSegment = `${sep}app.asar${sep}`;
	if (!__dirname.includes(asarSegment)) return __dirname;
	const unpackedDir = __dirname.replace(asarSegment, `${sep}app.asar.unpacked${sep}`);
	return existsSync(unpackedDir) ? unpackedDir : __dirname;
}

function nativeFileName() {
	if (process.platform !== 'win32') {
		throw new Error(`@fluxer/win-shell is only supported on Windows, got ${process.platform}`);
	}
	switch (process.arch) {
		case 'x64':
			return 'win-shell.win32-x64-msvc.node';
		case 'arm64':
			return 'win-shell.win32-arm64-msvc.node';
		default:
			throw new Error(`Unsupported Windows architecture: ${process.arch}`);
	}
}

let binding = null;
let loadError = null;

if (process.platform === 'win32') {
	try {
		const nativeRoot = resolveNativeRoot();
		const nativePath = join(nativeRoot, nativeFileName());
		const loaded = loadNativeBinding({
			moduleName: MODULE_NAME,
			nativePath,
			nativeRoot,
			packageDir: __dirname,
			probe: false,
		});
		binding = loaded.binding;
		loadError = loaded.loadError;
		if (loadError) throw loadError;
	} catch (error) {
		loadError = createNativeLoadError({
			moduleName: MODULE_NAME,
			nativeRoot: resolveNativeRoot(),
			packageDir: __dirname,
			reason: 'native loader threw before binding load completed',
			cause: error,
		});
		throw loadError;
	}
}

module.exports = {
	createShortcut: binding ? binding.createShortcut : null,
	setCurrentUserRunValue: binding ? binding.setCurrentUserRunValue : null,
	deleteCurrentUserRunValue: binding ? binding.deleteCurrentUserRunValue : null,
	getCurrentUserRunValue: binding ? binding.getCurrentUserRunValue : null,
	getUserNotificationState: binding ? binding.getUserNotificationState : null,
	loadError,
};
