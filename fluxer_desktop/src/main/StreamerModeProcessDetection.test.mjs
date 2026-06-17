// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {describe, test} from 'node:test';
import {fileURLToPath} from 'node:url';
import {promisify} from 'node:util';
import vm from 'node:vm';

const require = createRequire(import.meta.url);
const esbuild = require('esbuild');

const sourcePath = fileURLToPath(new URL('./StreamerModeProcessDetection.ts', import.meta.url));
const source = readFileSync(sourcePath, 'utf8');
const transformedSource = esbuild.transformSync(source, {
	loader: 'ts',
	format: 'cjs',
	platform: 'node',
	target: 'node20',
}).code;

function makeExecError(message, code) {
	const error = new Error(message);
	error.code = code;
	return error;
}

function plain(value) {
	return JSON.parse(JSON.stringify(value));
}

function loadStreamerModeProcessDetection({platform = 'linux', execFile}) {
	const calls = [];
	const execFileStub = (file, args, options, callback) => {
		calls.push({file, args, options});
		execFile(file, args, options, callback);
	};
	execFileStub[promisify.custom] = (file, args, options) =>
		new Promise((resolve, reject) => {
			execFileStub(file, args, options, (error, stdout, stderr) => {
				if (error) {
					error.stdout = stdout;
					error.stderr = stderr;
					reject(error);
					return;
				}
				resolve({stdout, stderr});
			});
		});
	const module = {exports: {}};
	const context = vm.createContext({
		module,
		exports: module.exports,
		console,
		process: {platform},
		require: (specifier) => {
			if (specifier === 'node:child_process') {
				return {
					execFile: execFileStub,
				};
			}
			return require(specifier);
		},
	});
	vm.runInContext(transformedSource, context, {filename: sourcePath});
	return {...module.exports, calls};
}

describe('StreamerModeProcessDetection', () => {
	test('detects POSIX capture apps with bounded candidate pgrep calls', async () => {
		const detector = loadStreamerModeProcessDetection({
			execFile: (_file, args, _options, callback) => {
				const processName = args.at(-1);
				if (processName === 'obs') {
					callback(null, '123\n', '');
					return;
				}
				callback(makeExecError('no matching processes', 1), '', '');
			},
		});

		const status = await detector.getStreamerModeCaptureAppStatus();

		assert.deepEqual(plain(status), {detected: true, processes: [{name: 'obs', pid: 123}]});
		assert.equal(
			detector.calls.every((call) => call.file === 'pgrep'),
			true,
		);
		assert.equal(
			detector.calls.every((call) => call.options.maxBuffer === 64 * 1024),
			true,
		);
	});

	test('falls back to compact POSIX ps query without command-line args', async () => {
		const detector = loadStreamerModeProcessDetection({
			execFile: (file, _args, _options, callback) => {
				if (file === 'pgrep') {
					callback(makeExecError('pgrep missing', 'ENOENT'), '', '');
					return;
				}
				callback(null, '  456 /Applications/OBS.app/Contents/MacOS/OBS\n  789 /bin/bash\n', '');
			},
		});

		const status = await detector.getStreamerModeCaptureAppStatus();

		assert.deepEqual(plain(status), {detected: true, processes: [{name: 'obs', pid: 456}]});
		assert.deepEqual(plain(detector.calls.at(-1).args), ['-axo', 'pid=,comm=']);
	});

	test('returns not detected when POSIX process query output exceeds maxBuffer', async () => {
		const detector = loadStreamerModeProcessDetection({
			execFile: (file, _args, _options, callback) => {
				if (file === 'pgrep') {
					callback(makeExecError('pgrep missing', 'ENOENT'), '', '');
					return;
				}
				callback(makeExecError('stdout maxBuffer length exceeded', 'ERR_CHILD_PROCESS_STDIO_MAXBUFFER'), '', '');
			},
		});

		const status = await detector.getStreamerModeCaptureAppStatus();

		assert.deepEqual(plain(status), {detected: false, processes: []});
	});

	test('uses filtered Windows tasklist probes', async () => {
		const detector = loadStreamerModeProcessDetection({
			platform: 'win32',
			execFile: (_file, args, _options, callback) => {
				const filter = args[1];
				if (filter === 'IMAGENAME eq obs64.exe') {
					callback(null, '"obs64.exe","321","Console","1","1,024 K"\r\n', '');
					return;
				}
				callback(null, 'INFO: No tasks are running which match the specified criteria.\r\n', '');
			},
		});

		const status = await detector.getStreamerModeCaptureAppStatus();

		assert.deepEqual(plain(status), {detected: true, processes: [{name: 'obs64.exe', pid: 321}]});
		assert.equal(
			detector.calls.every((call) => call.file === 'tasklist'),
			true,
		);
		assert.equal(
			detector.calls.every((call) => call.args[0] === '/fi'),
			true,
		);
	});
});
