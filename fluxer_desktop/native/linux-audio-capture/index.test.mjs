// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {test} from 'node:test';

const requireSrc = createRequire(import.meta.url);
const modulePath = requireSrc.resolve('./index.js');

function freshModule() {
	delete requireSrc.cache[modulePath];
	return requireSrc('./index.js');
}

function makeFakeBinding() {
	const captures = [];
	let startResult = true;
	class FakeDirectAudioCapture {
		constructor() {
			this.started = false;
			this.stopCount = 0;
			this.startRules = [];
			this.lifecycleCallback = undefined;
			captures.push(this);
		}

		setLifecycleCallback(callback) {
			this.lifecycleCallback = callback;
		}

		start(rule) {
			if (!startResult) return false;
			this.started = true;
			this.startRules.push(rule);
			return true;
		}

		setRule(rule) {
			this.startRules.push(rule);
			return true;
		}

		read() {
			return null;
		}

		stop() {
			this.started = false;
			this.stopCount += 1;
		}
	}

	function FakeAudioMixRuntimeHandle() {}
	FakeAudioMixRuntimeHandle.boundToDirectCapture = () => {
		throw new Error('ProcessLoopback must not tick a discard-only mix runtime');
	};

	return {
		binding: {
			AudioBridge: class {},
			AudioMixRuntimeHandle: FakeAudioMixRuntimeHandle,
			DirectAudioCapture: FakeDirectAudioCapture,
			pipeWireAvailable: () => true,
			audioBackend: () => 'pipewire',
		},
		captures,
		setStartResult(value) {
			startResult = value;
		},
	};
}

let loadError = null;
try {
	freshModule();
} catch (error) {
	loadError = error;
}

test('ProcessLoopback reuses idle direct captures after stop', {skip: loadError?.message}, async () => {
	const mod = freshModule();
	const {binding, captures} = makeFakeBinding();
	mod.__setBindingForTests(binding);

	const rule = {linuxRule: {include: [{'application.name': 'Firefox'}]}};
	const first = new mod.ProcessLoopback(rule);
	first.start();
	await first.stop();

	assert.equal(captures.length, 1);
	assert.equal(captures[0].stopCount, 1);
	assert.equal(mod.__getIdleDirectCaptureCountForTests(), 1);

	const second = new mod.ProcessLoopback(rule);
	second.start();
	await second.stop();

	assert.equal(captures.length, 1);
	assert.equal(captures[0].stopCount, 2);
	assert.equal(captures[0].startRules.length, 2);
	assert.equal(mod.__getIdleDirectCaptureCountForTests(), 1);
});

test('ProcessLoopback does not pool failed direct captures', {skip: loadError?.message}, async () => {
	const mod = freshModule();
	const {binding, captures, setStartResult} = makeFakeBinding();
	mod.__setBindingForTests(binding);

	const rule = {linuxRule: {include: [{'application.name': 'Firefox'}]}};
	setStartResult(false);
	const first = new mod.ProcessLoopback(rule);
	assert.throws(() => first.start(), /failed to start Linux direct audio capture/);
	await first.stop();

	assert.equal(captures.length, 1);
	assert.equal(mod.__getIdleDirectCaptureCountForTests(), 0);

	setStartResult(true);
	const second = new mod.ProcessLoopback(rule);
	second.start();
	await second.stop();

	assert.equal(captures.length, 2);
	assert.equal(mod.__getIdleDirectCaptureCountForTests(), 1);
});

test('ProcessLoopback installs native lifecycle callback on direct capture', {skip: loadError?.message}, () => {
	const mod = freshModule();
	const {binding, captures} = makeFakeBinding();
	mod.__setBindingForTests(binding);

	const loopback = new mod.ProcessLoopback({linuxRule: {include: [{'application.name': 'Firefox'}]}});

	assert.equal(captures.length, 1);
	assert.equal(typeof captures[0].lifecycleCallback, 'function');
	assert.equal(loopback.listenerCount('closed'), 0);
});

test('ProcessLoopback does not create a discard-only audio mix runtime', {skip: loadError?.message}, async () => {
	const mod = freshModule();
	const {binding} = makeFakeBinding();
	mod.__setBindingForTests(binding);

	const loopback = new mod.ProcessLoopback({linuxRule: {include: [{'application.name': 'Firefox'}]}});
	loopback.start();
	await new Promise((resolve) => setImmediate(resolve));
	await loopback.stop();
});

test('ProcessLoopback closes once when native lifecycle closes while idle', {skip: loadError?.message}, async () => {
	const mod = freshModule();
	const {binding, captures} = makeFakeBinding();
	mod.__setBindingForTests(binding);

	const loopback = new mod.ProcessLoopback({linuxRule: {include: [{'application.name': 'Firefox'}]}});
	let closed = 0;
	loopback.on('closed', () => {
		closed += 1;
	});

	captures[0].lifecycleCallback('closed-clean', 'daemon disconnected');
	await new Promise((resolve) => setImmediate(resolve));
	await loopback.stop();

	assert.equal(closed, 1);
	assert.equal(captures[0].stopCount, 1);
});
