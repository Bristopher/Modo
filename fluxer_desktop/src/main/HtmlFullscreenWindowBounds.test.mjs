// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./HtmlFullscreenWindowBounds.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadHtmlFullscreenWindowBounds() {
	const module = {exports: {}};
	const context = vm.createContext({
		module,
		exports: module.exports,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return module.exports;
}

const displayBounds = {x: 0, y: 0, width: 2560, height: 1440};
const compactBounds = {x: 320, y: 180, width: 1280, height: 720};

describe('HtmlFullscreenWindowBounds', () => {
	test('restores compact bounds when fullscreen exit leaves the window display-sized', () => {
		const {shouldRestoreHtmlFullscreenWindowBounds} = loadHtmlFullscreenWindowBounds();

		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: compactBounds,
				currentBounds: displayBounds,
				displayBounds,
				wasMaximized: false,
				isMaximized: false,
			}),
			true,
		);
	});

	test('does not restore when the previous window was already display-sized', () => {
		const {shouldRestoreHtmlFullscreenWindowBounds} = loadHtmlFullscreenWindowBounds();

		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: displayBounds,
				currentBounds: displayBounds,
				displayBounds,
				wasMaximized: false,
				isMaximized: false,
			}),
			false,
		);
	});

	test('does not restore when Electron already returned to compact bounds', () => {
		const {shouldRestoreHtmlFullscreenWindowBounds} = loadHtmlFullscreenWindowBounds();

		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: compactBounds,
				currentBounds: compactBounds,
				displayBounds,
				wasMaximized: false,
				isMaximized: false,
			}),
			false,
		);
	});

	test('does not restore maximized windows', () => {
		const {shouldRestoreHtmlFullscreenWindowBounds} = loadHtmlFullscreenWindowBounds();

		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: compactBounds,
				currentBounds: displayBounds,
				displayBounds,
				wasMaximized: true,
				isMaximized: false,
			}),
			false,
		);
		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: compactBounds,
				currentBounds: displayBounds,
				displayBounds,
				wasMaximized: false,
				isMaximized: true,
			}),
			false,
		);
	});

	test('allows native window-frame tolerance around display bounds', () => {
		const {shouldRestoreHtmlFullscreenWindowBounds} = loadHtmlFullscreenWindowBounds();

		assert.equal(
			shouldRestoreHtmlFullscreenWindowBounds({
				previousBounds: compactBounds,
				currentBounds: {x: -1, y: -1, width: 2561, height: 1441},
				displayBounds,
				wasMaximized: false,
				isMaximized: false,
			}),
			true,
		);
	});
});
