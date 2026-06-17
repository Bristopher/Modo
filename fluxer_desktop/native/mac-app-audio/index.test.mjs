// SPDX-License-Identifier: AGPL-3.0-or-later

import {createRequire} from 'node:module';
import {afterEach, describe, expect, test} from 'vitest';

const require = createRequire(import.meta.url);
const macAppAudio = require('./index.js');

function makeFakeBinding() {
	const calls = [];
	const natives = [];
	class FakeNative {
		constructor() {
			this.frameCallback = undefined;
			this.lifecycleCallback = undefined;
			this.stopCount = 0;
			natives.push(this);
		}

		setFrameCallback(callback) {
			this.frameCallback = callback;
		}

		setLifecycleCallback(callback) {
			this.lifecycleCallback = callback;
		}

		async start(pid, excludeSelf, includeProcessTree, backend, captureScope) {
			calls.push({pid, excludeSelf, includeProcessTree, backend, captureScope});
		}

		async stop() {
			this.stopCount += 1;
		}
	}
	return {
		binding: {
			ProcessLoopback: FakeNative,
			listAudibleApplications: async () => [],
			getBackendAvailability: async () => ({}),
			pidFromWindowId: () => 0,
		},
		calls,
		natives,
	};
}

afterEach(() => {
	macAppAudio.__setBindingForTests(null);
});

describe('mac-app-audio loader wrapper', () => {
	test('forces excludeSelf=true and preserves includeProcessTree=false', async () => {
		const {binding, calls} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234, {
			excludeSelf: false,
			includeProcessTree: false,
		});
		loopback.on('error', () => {});
		await loopback.start();
		expect(calls).toEqual([
			{pid: 1234, excludeSelf: true, includeProcessTree: false, backend: 'auto', captureScope: 'process'},
		]);
	});
	test('forwards explicit backend preference to native binding', async () => {
		const {binding, calls} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234, {
			backend: 'coreaudio',
		});
		loopback.on('error', () => {});
		await loopback.start();
		expect(calls).toEqual([
			{pid: 1234, excludeSelf: true, includeProcessTree: true, backend: 'coreaudio', captureScope: 'process'},
		]);
	});
	test('forwards explicit capture scope to native binding', async () => {
		const {binding, calls} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234, {
			captureScope: 'system',
			backend: 'coreaudio',
		});
		loopback.on('error', () => {});
		await loopback.start();
		expect(calls).toEqual([
			{pid: 1234, excludeSelf: true, includeProcessTree: true, backend: 'coreaudio', captureScope: 'system'},
		]);
	});
	test('forwards ScreenCaptureKit system capture requests to native binding', async () => {
		const {binding, calls} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234, {
			captureScope: 'system',
			backend: 'sck',
		});
		loopback.on('error', () => {});
		await loopback.start();
		expect(calls).toEqual([
			{pid: 1234, excludeSelf: true, includeProcessTree: true, backend: 'sck', captureScope: 'system'},
		]);
	});
	test('copies native frame samples before emitting', () => {
		const {binding, natives} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234);
		const frames = [];
		loopback.on('frame', (frame) => frames.push(frame));
		const nativeSamples = new Float32Array([0.1, 0.2, 0.3, 0.4]);
		natives[0].frameCallback(nativeSamples);
		nativeSamples.fill(9);
		expect(frames).toHaveLength(1);
		expect(Array.from(frames[0].samples)).toEqual(Array.from(new Float32Array([0.1, 0.2, 0.3, 0.4])));
		expect(frames[0].channels).toBe(2);
		expect(frames[0].sampleRate).toBe(48000);
	});
	test('emits closed once when native lifecycle closes and stop is called later', async () => {
		const {binding, natives} = makeFakeBinding();
		macAppAudio.__setBindingForTests(binding);
		const loopback = new macAppAudio.ProcessLoopback(1234);
		let closed = 0;
		loopback.on('closed', () => {
			closed += 1;
		});
		natives[0].lifecycleCallback('closed', '');
		await Promise.resolve();
		await loopback.stop();
		expect(closed).toBe(1);
		expect(natives[0].stopCount).toBe(1);
	});
});
