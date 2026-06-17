// SPDX-License-Identifier: AGPL-3.0-or-later

const {existsSync} = require('node:fs');
const {join, sep} = require('node:path');
const {createNativeLoadError, loadNativeBinding} = require('./loader-diagnostics.cjs');
const MODULE_NAME = '@fluxer/win-toast';

function resolveNativeRoot() {
	const asarSegment = `${sep}app.asar${sep}`;
	if (!__dirname.includes(asarSegment)) return __dirname;
	const unpackedDir = __dirname.replace(asarSegment, `${sep}app.asar.unpacked${sep}`);
	return existsSync(unpackedDir) ? unpackedDir : __dirname;
}

function nativeFileName() {
	if (process.platform !== 'win32') {
		throw new Error(`@fluxer/win-toast is only supported on Windows, got ${process.platform}`);
	}
	switch (process.arch) {
		case 'x64':
			return 'win-toast.win32-x64-msvc.node';
		case 'arm64':
			return 'win-toast.win32-arm64-msvc.node';
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

const stubSupport = () => ({supported: false, reason: loadError ? String(loadError.message) : 'addon not loaded'});
const stubAsync = () => Promise.reject(loadError ?? new Error('@fluxer/win-toast not loaded'));

module.exports = {
	isSupported: binding ? binding.isSupported : stubSupport,
	notify: binding ? binding.notify : stubAsync,
	dismiss: binding ? binding.dismiss : stubAsync,
	clear: binding ? binding.clear : stubAsync,
	loadError,
};
