// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./WindowRendererLifecycle.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadWindowRendererLifecycle() {
	const module = {exports: {}};
	const context = vm.createContext({
		module,
		exports: module.exports,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return module.exports;
}

const visibleMacContext = {
	platform: 'darwin',
	isQuitting: false,
	isMainWindowHidden: false,
	closeToTrayEnabled: true,
	reloadedRecently: false,
};

const hiddenMacCloseToTrayContext = {
	...visibleMacContext,
	isMainWindowHidden: true,
};

describe('WindowRendererLifecycle', () => {
	test('reloads a hidden macOS close-to-tray window instead of quitting after a killed renderer', () => {
		const {getMainWindowRendererGoneAction} = loadWindowRendererLifecycle();

		assert.equal(getMainWindowRendererGoneAction({reason: 'killed'}, hiddenMacCloseToTrayContext), 'reload');
	});

	test('reloads a hidden macOS close-to-tray window after a clean renderer exit', () => {
		const {getMainWindowRendererGoneAction} = loadWindowRendererLifecycle();

		assert.equal(getMainWindowRendererGoneAction({reason: 'clean-exit'}, hiddenMacCloseToTrayContext), 'reload');
	});

	test('defers repeated hidden macOS close-to-tray recovery instead of quitting', () => {
		const {getMainWindowRendererGoneAction} = loadWindowRendererLifecycle();

		assert.equal(
			getMainWindowRendererGoneAction(
				{reason: 'killed'},
				{
					...hiddenMacCloseToTrayContext,
					reloadedRecently: true,
				},
			),
			'defer-reload',
		);
	});

	test('defers repeated hidden macOS crash recovery instead of quitting', () => {
		const {getMainWindowRendererGoneAction} = loadWindowRendererLifecycle();

		assert.equal(
			getMainWindowRendererGoneAction(
				{reason: 'crashed'},
				{
					...hiddenMacCloseToTrayContext,
					reloadedRecently: true,
				},
			),
			'defer-reload',
		);
	});

	test('keeps the existing visible killed-renderer quit behavior', () => {
		const {getMainWindowRendererGoneAction} = loadWindowRendererLifecycle();

		assert.equal(getMainWindowRendererGoneAction({reason: 'killed'}, visibleMacContext), 'quit');
	});
});
