// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./NativeScreenCaptureValidation.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const fixturePath = fileURLToPath(new URL('./fixtures/native_screen_capture_validation.json', import.meta.url));
const validationFixtures = JSON.parse(readFileSync(fixturePath, 'utf8'));
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadValidationModule() {
	const module = {exports: {}};
	const context = vm.createContext({
		exports: module.exports,
		module,
		Object,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return module.exports;
}

describe('NativeScreenCaptureValidation', () => {
	for (const fixture of validationFixtures) {
		test(`replays fixture: ${fixture.name}`, () => {
			const {isValidStartOptions} = loadValidationModule();
			assert.equal(isValidStartOptions(fixture.options), fixture.expected);
		});
	}

	test('accepts optional cursor click and capture rect start options', () => {
		const {isValidStartOptions} = loadValidationModule();
		assert.equal(
			isValidStartOptions({
				sourceId: '42',
				sourceKind: 'window',
				showCursorClicks: true,
				captureRect: {x: 10, y: 20, width: 300, height: 200},
				nativeFrameSinkRequired: true,
			}),
			true,
		);
	});

	test('rejects malformed cursor click, capture rect, and non-zero-copy options', () => {
		const {isValidStartOptions} = loadValidationModule();
		const base = {sourceId: '42', sourceKind: 'window', nativeFrameSinkRequired: true};

		assert.equal(isValidStartOptions({...base, showCursorClicks: 'true'}), false);
		assert.equal(isValidStartOptions({...base, captureRect: {x: 0, y: 0, width: 0, height: 100}}), false);
		assert.equal(isValidStartOptions({...base, captureRect: {x: 0.5, y: 0, width: 100, height: 100}}), false);
		assert.equal(isValidStartOptions({...base, captureRect: {x: 0, y: 0, width: 9000, height: 100}}), false);
		assert.equal(isValidStartOptions({sourceId: '42', sourceKind: 'window'}), false);
		assert.equal(isValidStartOptions({...base, nativeFrameSinkRequired: false}), false);
	});
});
