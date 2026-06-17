// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./GlobalKeyHookLifecycle.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadGlobalKeyHookLifecycle() {
	const module = {exports: {}};
	const context = vm.createContext({
		module,
		exports: module.exports,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return module.exports;
}

const {GlobalKeyHookLifecycle} = loadGlobalKeyHookLifecycle();

function createBackend({startResults = [true]} = {}) {
	const calls = [];
	let startIndex = 0;
	return {
		calls,
		backend: {
			start() {
				calls.push('start');
				const result = startResults[Math.min(startIndex, startResults.length - 1)];
				startIndex += 1;
				return Promise.resolve(result);
			},
			stop() {
				calls.push('stop');
			},
		},
	};
}

describe('GlobalKeyHookLifecycle', () => {
	test('first acquire starts the backend once and reports running', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		assert.equal(await lifecycle.acquire(1), true);
		assert.equal(await lifecycle.acquire(1), true);
		assert.deepEqual(calls, ['start']);
		assert.equal(lifecycle.isRunning(), true);
		assert.equal(lifecycle.acquisitionCount(), 2);
	});

	test('release keeps the backend running while other acquisitions remain', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		await lifecycle.acquire(1);
		await lifecycle.acquire(1);
		await lifecycle.release(1);
		assert.deepEqual(calls, ['start']);
		assert.equal(lifecycle.isRunning(), true);
		await lifecycle.release(1);
		assert.deepEqual(calls, ['start', 'stop']);
		assert.equal(lifecycle.isRunning(), false);
	});

	test('recorder and runtime sharing one owner survive the recorder closing', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		assert.equal(await lifecycle.acquire(7), true);
		assert.equal(await lifecycle.acquire(7), true);
		await lifecycle.release(7);
		assert.equal(lifecycle.isRunning(), true);
		assert.deepEqual(calls, ['start']);
	});

	test('failed start does not record an acquisition and allows retry', async () => {
		const {backend, calls} = createBackend({startResults: [false, true]});
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		assert.equal(await lifecycle.acquire(1), false);
		assert.equal(lifecycle.isRunning(), false);
		assert.equal(lifecycle.acquisitionCount(), 0);
		assert.equal(await lifecycle.acquire(1), true);
		assert.equal(lifecycle.isRunning(), true);
		assert.deepEqual(calls, ['start', 'start']);
	});

	test('start that throws is treated as a failed acquire', async () => {
		const lifecycle = new GlobalKeyHookLifecycle({
			start() {
				return Promise.reject(new Error('boom'));
			},
			stop() {},
		});
		assert.equal(await lifecycle.acquire(1), false);
		assert.equal(lifecycle.isRunning(), false);
	});

	test('release without a matching acquire is a no-op', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		await lifecycle.release(1);
		assert.deepEqual(calls, []);
		await lifecycle.acquire(1);
		await lifecycle.release(2);
		assert.equal(lifecycle.isRunning(), true);
		await lifecycle.release(1);
		assert.equal(lifecycle.isRunning(), false);
		assert.deepEqual(calls, ['start', 'stop']);
	});

	test('releaseAllForOwner drops only that owner', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		await lifecycle.acquire(1);
		await lifecycle.acquire(1);
		await lifecycle.acquire(2);
		await lifecycle.releaseAllForOwner(1);
		assert.equal(lifecycle.isRunning(), true);
		assert.equal(lifecycle.acquisitionCount(), 1);
		await lifecycle.releaseAllForOwner(2);
		assert.equal(lifecycle.isRunning(), false);
		assert.deepEqual(calls, ['start', 'stop']);
	});

	test('forceStop clears every acquisition and stops the backend', async () => {
		const {backend, calls} = createBackend();
		const lifecycle = new GlobalKeyHookLifecycle(backend);
		await lifecycle.acquire(1);
		await lifecycle.acquire(2);
		await lifecycle.forceStop();
		assert.equal(lifecycle.isRunning(), false);
		assert.equal(lifecycle.acquisitionCount(), 0);
		assert.deepEqual(calls, ['start', 'stop']);
	});

	test('operations issued while a slow start is pending stay ordered', async () => {
		const calls = [];
		let resolveStart;
		const lifecycle = new GlobalKeyHookLifecycle({
			start() {
				calls.push('start');
				return new Promise((resolve) => {
					resolveStart = resolve;
				});
			},
			stop() {
				calls.push('stop');
			},
		});
		const first = lifecycle.acquire(1);
		const second = lifecycle.acquire(1);
		const release = lifecycle.release(1);
		const releaseLast = lifecycle.release(1);
		while (resolveStart === undefined) {
			await Promise.resolve();
		}
		resolveStart(true);
		assert.equal(await first, true);
		assert.equal(await second, true);
		await release;
		await releaseLast;
		assert.equal(lifecycle.isRunning(), false);
		assert.deepEqual(calls, ['start', 'stop']);
	});
});
