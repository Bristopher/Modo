// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import {createRequire} from 'node:module';
import {test} from 'node:test';

const requireSrc = createRequire(import.meta.url);
const modulePath = requireSrc.resolve('./index.js');

function freshModule() {
	delete requireSrc.cache[modulePath];
	return requireSrc('./index.js');
}

test('module loads without throwing on non-Windows hosts', () => {
	const mod = freshModule();
	assert.equal(typeof mod.isSupported, 'function');
	assert.equal(typeof mod.pidFromHwnd, 'function');
	assert.equal(typeof mod.resolveAudioRootPid, 'function');
	assert.equal(typeof mod.ProcessLoopback, 'function');
});

test('non-Windows hosts surface a load error', {skip: process.platform === 'win32'}, () => {
	const mod = freshModule();
	assert.ok(mod.loadError instanceof Error, 'expected loadError to be set on non-Windows');
	assert.match(mod.loadError.message, /not supported on platform/i);
});

test('isSupported / pidFromHwnd / resolveAudioRootPid fail soft when binding is absent', () => {
	const mod = freshModule();
	mod.__setBindingForTests(null, new Error('test: binary missing'));
	assert.equal(mod.isSupported(), false);
	assert.equal(mod.pidFromHwnd(123n), 0);
	assert.equal(mod.resolveAudioRootPid(1234), 1234);
	assert.deepEqual(
		{
			processSupported: mod.getBackendInfo().processSupported,
			systemSupported: mod.getBackendInfo().systemSupported,
			sessionMixerSupported: mod.getBackendInfo().sessionMixerSupported,
			systemLoopbackMode: mod.getBackendInfo().systemLoopbackMode,
		},
		{
			processSupported: false,
			systemSupported: false,
			sessionMixerSupported: false,
			systemLoopbackMode: 'unavailable',
		},
	);
});

test('constructing ProcessLoopback without a binding throws the recorded loadError', () => {
	const mod = freshModule();
	const recorded = new Error('synthetic: binary unavailable');
	mod.__setBindingForTests(null, recorded);
	assert.throws(
		() => new mod.ProcessLoopback(1234, {}),
		(error) => error === recorded,
	);
});

test('isSupported swallows binding-side exceptions and returns false', () => {
	const mod = freshModule();
	mod.__setBindingForTests({
		isSupported: () => {
			throw new Error('binding crashed during isSupported');
		},
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeProcessLoopback() {},
	});
	assert.equal(mod.isSupported(), false);
});

test('ProcessLoopback wires native frame/error/closed events to the EventEmitter', async () => {
	const mod = freshModule();
	let frameCb;
	let errorCb;
	let closedCb;
	let startedCb;
	class FakeNative {
		constructor(pid, opts, onFrame, onError, onClosed, onStarted) {
			this.pid = pid;
			this.opts = opts;
			frameCb = onFrame;
			errorCb = onError;
			closedCb = onClosed;
			startedCb = onStarted;
		}

		start() {
			this.started = true;
		}

		dispose() {
			this.disposed = true;
		}
	}
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: FakeNative,
	});
	const loopback = new mod.ProcessLoopback(4242, {includeProcessTree: false});
	assert.ok(loopback instanceof EventEmitter);
	const frames = [];
	const errors = [];
	let closedCount = 0;
	loopback.on('frame', (frame) => frames.push(frame));
	loopback.on('error', (error) => errors.push(error));
	loopback.on('closed', () => closedCount++);
	const startPromise = loopback.start();
	startedCb();
	await startPromise;
	frameCb({samples: new Float32Array(2), sampleRate: 48000, channels: 2, timestampUs: 0});
	errorCb(new Error('boom'));
	closedCb();
	closedCb();
	assert.equal(frames.length, 1);
	assert.equal(errors.length, 1);
	assert.equal(closedCount, 1);
});

test('ProcessLoopback.start waits for the native started callback', async () => {
	const mod = freshModule();
	let startedCb;
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative(_pid, _opts, _onFrame, _onError, _onClosed, onStarted) {
			startedCb = onStarted;
			return {start() {}, dispose() {}};
		},
	});
	const loopback = new mod.ProcessLoopback(4242);
	let resolved = false;
	const startPromise = loopback.start().then(() => {
		resolved = true;
	});
	await Promise.resolve();
	assert.equal(resolved, false);
	startedCb();
	await startPromise;
	assert.equal(resolved, true);
});

test('ProcessLoopback.start rejects if native capture errors before start', async () => {
	const mod = freshModule();
	let errorCb;
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative(_pid, _opts, _onFrame, onError) {
			errorCb = onError;
			return {start() {}, dispose() {}};
		},
	});
	const loopback = new mod.ProcessLoopback(4242);
	loopback.on('error', () => {});
	const startPromise = loopback.start();
	const error = new Error('activation failed');
	errorCb(error);
	await assert.rejects(startPromise, (actual) => actual === error);
});

test('opts default includeProcessTree to true and forward sample/channel options', () => {
	const mod = freshModule();
	let observedOpts;
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative(_pid, opts) {
			observedOpts = opts;
			return {start() {}, dispose() {}};
		},
	});
	const ctor = mod.ProcessLoopback;
	void new ctor(1, {sampleRate: 44100, channels: 1});
	assert.deepEqual(observedOpts, {includeProcessTree: true, sampleRate: 44100, channels: 1, captureScope: 'process'});
	void new ctor(1, {includeProcessTree: false});
	assert.equal(observedOpts.includeProcessTree, false);
});

test('system capture scope forwards EXCLUDE target process tree mode', () => {
	const mod = freshModule();
	let observedOpts;
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative(_pid, opts) {
			observedOpts = opts;
			return {start() {}, dispose() {}};
		},
	});
	void new mod.ProcessLoopback(1, {captureScope: 'system', includeProcessTree: true});
	assert.deepEqual(observedOpts, {
		includeProcessTree: false,
		sampleRate: undefined,
		channels: undefined,
		captureScope: 'system',
	});
});

test('session mixer capture scope is forwarded and never captures the target process tree directly', () => {
	const mod = freshModule();
	let observedOpts;
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative(_pid, opts) {
			observedOpts = opts;
			return {start() {}, dispose() {}};
		},
	});
	void new mod.ProcessLoopback(1, {captureScope: 'session-mixer', includeProcessTree: true});
	assert.deepEqual(observedOpts, {
		includeProcessTree: false,
		sampleRate: undefined,
		channels: undefined,
		captureScope: 'session-mixer',
	});
});

function loopbackWithNativeSinkResult(setScreenAudioSink) {
	const mod = freshModule();
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative() {
			return {start() {}, dispose() {}, setScreenAudioSink};
		},
	});
	return new mod.ProcessLoopback(4242);
}

test('setScreenAudioSink reports true when the native attach succeeds', () => {
	const loopback = loopbackWithNativeSinkResult(() => undefined);
	assert.equal(loopback.setScreenAudioSink({}), true);
});

test('setScreenAudioSink reports false when the native attach returns false', () => {
	const loopback = loopbackWithNativeSinkResult(() => false);
	assert.equal(loopback.setScreenAudioSink({}), false);
});

test('setScreenAudioSink reports false when the native attach throws', () => {
	const loopback = loopbackWithNativeSinkResult(() => {
		throw new Error('attach failed');
	});
	assert.equal(loopback.setScreenAudioSink({}), false);
});

test('setScreenAudioSink reports false when the native method is missing', () => {
	const mod = freshModule();
	mod.__setBindingForTests({
		isSupported: () => true,
		pidFromHwnd: () => 0,
		resolveAudioRootPid: (pid) => pid,
		ProcessLoopback: function FakeNative() {
			return {start() {}, dispose() {}};
		},
	});
	const loopback = new mod.ProcessLoopback(4242);
	assert.equal(loopback.setScreenAudioSink({}), false);
});
