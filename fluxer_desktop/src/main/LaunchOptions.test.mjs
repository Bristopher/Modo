// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./LaunchOptions.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadLaunchOptions() {
	const module = {exports: {}};
	const context = vm.createContext({
		module,
		exports: module.exports,
		process: {env: {}, platform: 'linux'},
		console,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return module.exports;
}

describe('LaunchOptions start hidden at login', () => {
	test('never starts hidden at login', () => {
		const {shouldStartHiddenAtLogin} = loadLaunchOptions();

		assert.equal(shouldStartHiddenAtLogin(), false);
	});
});
