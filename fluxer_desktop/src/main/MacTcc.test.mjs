// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./MacTcc.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function loadMacTcc({platform = 'darwin', addon = null, addonError = null, mediaAccessStatus = 'granted'} = {}) {
	const handlers = new Map();
	const mediaAccessCalls = [];

	function requireStub(specifier) {
		if (specifier === 'node:module') {
			return {
				createRequire: () => (moduleSpecifier) => {
					if (moduleSpecifier === '@fluxer/mac-tcc') {
						if (addonError) throw addonError;
						if (!addon) throw new Error('No fake addon configured for @fluxer/mac-tcc');
						return addon;
					}
					throw new Error(`Unexpected createRequire import: ${moduleSpecifier}`);
				},
			};
		}
		if (specifier === '@electron/common/Logger') {
			return {
				createChildLogger: () => ({
					info: () => {},
					warn: () => {},
				}),
			};
		}
		if (specifier === 'electron') {
			return {
				ipcMain: {
					handle(channel, handler) {
						handlers.set(channel, handler);
					},
					removeHandler(channel) {
						handlers.delete(channel);
					},
				},
				systemPreferences: {
					getMediaAccessStatus(type) {
						mediaAccessCalls.push(type);
						return mediaAccessStatus;
					},
				},
			};
		}
		throw new Error(`Unexpected import: ${specifier}`);
	}

	const module = {exports: {}};
	const context = vm.createContext({
		exports: module.exports,
		module,
		process: {env: {}, platform},
		require: requireStub,
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});

	return {handlers, mediaAccessCalls, module: module.exports};
}

function makeAddon(overrides = {}) {
	return {
		screenRecordingStatus: () => 'denied',
		requestScreenRecording: () => 'denied',
		inputMonitoringStatus: () => 'denied',
		requestInputMonitoring: () => 'denied',
		loadError: null,
		...overrides,
	};
}

describe('MacTcc', () => {
	test('reads statuses from the addon when it loads', () => {
		const {module, mediaAccessCalls} = loadMacTcc({
			addon: makeAddon({
				screenRecordingStatus: () => 'granted',
				inputMonitoringStatus: () => 'denied',
			}),
		});
		assert.equal(module.getTccStatus('screen-recording'), 'granted');
		assert.equal(module.getTccStatus('input-monitoring'), 'denied');
		assert.deepEqual(mediaAccessCalls, []);
	});

	test('falls back to systemPreferences for screen recording when the addon is missing', () => {
		const {module, mediaAccessCalls} = loadMacTcc({
			addonError: new Error('addon not built'),
			mediaAccessStatus: 'denied',
		});
		assert.equal(module.getTccStatus('screen-recording'), 'denied');
		assert.deepEqual(mediaAccessCalls, ['screen']);
	});

	test('maps restricted media access to denied in the screen fallback', () => {
		const {module} = loadMacTcc({
			addonError: new Error('addon not built'),
			mediaAccessStatus: 'restricted',
		});
		assert.equal(module.getTccStatus('screen-recording'), 'denied');
	});

	test('reports not-determined for input monitoring when the addon is missing', () => {
		const {module, mediaAccessCalls} = loadMacTcc({
			addonError: new Error('addon not built'),
		});
		assert.equal(module.getTccStatus('input-monitoring'), 'not-determined');
		assert.deepEqual(mediaAccessCalls, []);
	});

	test('treats an addon load error like a missing addon', () => {
		const {module, mediaAccessCalls} = loadMacTcc({
			addon: makeAddon({loadError: new Error('dlopen failed')}),
			mediaAccessStatus: 'granted',
		});
		assert.equal(module.getTccStatus('screen-recording'), 'granted');
		assert.deepEqual(mediaAccessCalls, ['screen']);
	});

	test('never touches systemPreferences off macOS', () => {
		const {module, mediaAccessCalls} = loadMacTcc({platform: 'linux'});
		assert.equal(module.getTccStatus('screen-recording'), 'not-determined');
		assert.equal(module.getTccStatus('input-monitoring'), 'not-determined');
		assert.deepEqual(mediaAccessCalls, []);
	});

	test('request handlers fall back to the status path when the addon is missing', () => {
		const {handlers, module, mediaAccessCalls} = loadMacTcc({
			addonError: new Error('addon not built'),
			mediaAccessStatus: 'denied',
		});
		module.registerMacTccIpcHandlers();
		const request = handlers.get('mac-tcc:request');
		assert.equal(request(null, 'screen-recording'), 'denied');
		assert.equal(request(null, 'input-monitoring'), 'not-determined');
		assert.deepEqual(mediaAccessCalls, ['screen']);
	});
});
