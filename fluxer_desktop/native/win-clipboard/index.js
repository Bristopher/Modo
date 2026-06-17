// SPDX-License-Identifier: AGPL-3.0-or-later

const {existsSync} = require('node:fs');
const {join, sep} = require('node:path');
const {createNativeLoadError, loadNativeBinding} = require('./loader-diagnostics.cjs');
const MODULE_NAME = '@fluxer/win-clipboard';

function resolveNativeRoot() {
	const asarSegment = `${sep}app.asar${sep}`;
	if (!__dirname.includes(asarSegment)) return __dirname;
	const unpackedDir = __dirname.replace(asarSegment, `${sep}app.asar.unpacked${sep}`);
	return existsSync(unpackedDir) ? unpackedDir : __dirname;
}

function nativeFileName() {
	if (process.platform !== 'win32') {
		throw new Error(`@fluxer/win-clipboard is only supported on Windows, got ${process.platform}`);
	}
	switch (process.arch) {
		case 'x64':
			return 'win-clipboard.win32-x64-msvc.node';
		case 'arm64':
			return 'win-clipboard.win32-arm64-msvc.node';
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
	writeFileReferenceToClipboard: binding ? binding.writeFileReferenceToClipboard : null,
	loadError,
};
