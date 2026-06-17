#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later

import {spawn} from 'node:child_process';
import {existsSync} from 'node:fs';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

const repoDir = new URL('..', import.meta.url).pathname;
const workspaceDir = path.resolve(repoDir, '..');
const args = new Set(process.argv.slice(2));
const mode = args.has('--strict')
	? 'strict'
	: args.has('--smoke')
		? 'smoke'
		: (process.env.FLUXER_NATIVE_MEDIA_MODE || 'smoke').toLowerCase();
const strict = mode === 'strict';
const packageManager = process.env.FLUXER_NATIVE_MEDIA_PACKAGE_MANAGER || 'pnpm';
const reportDir =
	process.env.FLUXER_NATIVE_MEDIA_REPORT_DIR ||
	path.join(repoDir, 'native-media-reports', new Date().toISOString().replace(/[:.]/g, '-'));
const livekitEnabled = strict || process.env.FLUXER_NATIVE_MEDIA_LIVEKIT !== '0';
const electronBuildEnabled = process.env.FLUXER_NATIVE_MEDIA_ELECTRON_BUILD === '1' || strict;
const strictCodecMatrix = ['vp8', 'vp9', 'h264', 'hevc', 'av1'];
const strictLiveKitFeatureFlags = [
	'LIVEKIT_ENABLE_SECOND_PUBLISHER',
	'LIVEKIT_ENABLE_MICROPHONE',
	'LIVEKIT_ENABLE_SCREEN_AUDIO',
	'LIVEKIT_ENABLE_CAMERA',
	'LIVEKIT_ENABLE_DATA_PACKET',
	'LIVEKIT_ENABLE_SUBSCRIPTION_CYCLE',
];
const strictLiveKitCredentialGroups = [
	['LIVEKIT_URL', 'LIVEKIT_WS_URL'],
	['LIVEKIT_API_KEY'],
	['LIVEKIT_API_SECRET', 'LIVEKIT_SECRET'],
];
const validModes = new Set(['smoke', 'strict']);

function scriptCommand(directory, scriptName) {
	return `${packageManager} --dir ${directory} ${scriptName}`;
}

function shellQuote(value) {
	if (process.platform === 'win32') return `"${String(value).replace(/"/g, '""')}"`;
	return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function findToolBinary(name) {
	const executable = process.platform === 'win32' ? `${name}.cmd` : name;
	const candidates = [
		path.join(repoDir, 'node_modules', '.bin', executable),
		path.join(workspaceDir, 'node_modules', '.bin', executable),
		path.join(workspaceDir, 'node_modules', '.pnpm', 'node_modules', '.bin', executable),
	];
	const found = candidates.find((candidate) => existsSync(candidate));
	return found ? shellQuote(found) : name;
}

function withDefaultEnv(defaults) {
	const env = {...process.env};
	if (env.CARGO_INCREMENTAL === undefined || env.CARGO_INCREMENTAL === '') {
		env.CARGO_INCREMENTAL = '0';
	}
	for (const [key, value] of Object.entries(defaults)) {
		if (env[key] === undefined || env[key] === '') {
			env[key] = String(value);
		}
	}
	return env;
}

function commandEnv(defaults, overrides = {}) {
	const env = withDefaultEnv(defaults);
	for (const [key, value] of Object.entries(overrides)) {
		env[key] = String(value);
	}
	return env;
}

function envExplicitFalse(name) {
	const value = process.env[name];
	return value !== undefined && /^(0|false|no|off|disabled|skip)$/i.test(value.trim());
}

function envPresent(names) {
	return names.some((name) => process.env[name]?.trim());
}

function parseCodecEnv(name) {
	const value = process.env[name];
	if (!value?.trim()) return null;
	return value
		.split(',')
		.map((entry) => entry.trim().toLowerCase())
		.filter(Boolean);
}

function codecListIncludes(codecs, codec) {
	if (codec === 'hevc') return codecs.includes('hevc') || codecs.includes('h265');
	return codecs.includes(codec);
}

function platformNativeCommands(platform = process.platform) {
	if (platform === 'linux') {
		return [
			{
				name: 'linux-screen-capture-build',
				command: scriptCommand('native/linux-screen-capture', 'build'),
				category: 'platform-native',
			},
			{
				name: 'linux-screen-capture-tests',
				command: scriptCommand('native/linux-screen-capture', 'test'),
				category: 'platform-native',
			},
			{
				name: 'linux-audio-capture-build',
				command: scriptCommand('native/linux-audio-capture', 'build'),
				category: 'platform-native',
			},
			{
				name: 'linux-audio-capture-tests',
				command: scriptCommand('native/linux-audio-capture', 'test'),
				category: 'platform-native',
			},
		];
	}

	if (platform === 'darwin') {
		return [
			{
				name: 'mac-screen-capture-build',
				command: scriptCommand('native/mac-screen-capture', 'build'),
				category: 'platform-native',
			},
			{
				name: 'mac-screen-capture-tests',
				command: scriptCommand('native/mac-screen-capture', 'test'),
				category: 'platform-native',
			},
			{
				name: 'mac-app-audio-build',
				command: scriptCommand('native/mac-app-audio', 'build'),
				category: 'platform-native',
			},
			{
				name: 'mac-app-audio-rust-tests',
				command: scriptCommand('native/mac-app-audio', 'test:cargo'),
				category: 'platform-native',
			},
			{
				name: 'mac-app-audio-js-tests',
				command: `${findToolBinary('vitest')} run`,
				cwd: 'native/mac-app-audio',
				category: 'platform-native',
			},
		];
	}

	if (platform === 'win32') {
		return [
			{
				name: 'win-game-capture-build',
				command: scriptCommand('native/win-game-capture', 'build'),
				category: 'platform-native',
			},
			{
				name: 'win-game-capture-tests',
				command: scriptCommand('native/win-game-capture', 'test'),
				category: 'platform-native',
			},
			{
				name: 'win-game-capture-fixtures',
				command: scriptCommand('native/win-game-capture', 'test:fixtures'),
				category: 'platform-native',
			},
			{
				name: 'win-process-loopback-build',
				command: scriptCommand('native/win-process-loopback', 'build'),
				category: 'platform-native',
			},
			{
				name: 'win-process-loopback-tests',
				command: scriptCommand('native/win-process-loopback', 'test'),
				category: 'platform-native',
			},
		];
	}

	return [];
}

function commandPlan() {
	const commands = [
		{
			name: 'main-process-native-media-unit-tests',
			command: 'node --test src/main/NativeVoiceEngine.test.mjs src/main/NativeScreenCapture.test.mjs',
			category: 'shared',
		},
		{name: 'desktop-typecheck', command: `${packageManager} typecheck`, category: 'shared'},
		{name: 'webrtc-sender-build', command: scriptCommand('native/webrtc-sender', 'build'), category: 'native-sender'},
		{name: 'webrtc-sender-tests', command: scriptCommand('native/webrtc-sender', 'test'), category: 'native-sender'},
	];

	if (livekitEnabled) {
		commands.push({
			name: 'webrtc-sender-livekit-harness',
			command: scriptCommand('native/webrtc-sender', 'test:livekit'),
			category: 'livekit',
			env: commandEnv(
				{
					LIVEKIT_HARNESS_STRICT: strict ? '1' : '0',
					LIVEKIT_ENABLE_SECOND_PUBLISHER: strict ? '1' : '0',
					LIVEKIT_ENABLE_SCREEN_AUDIO: strict ? '1' : '0',
					LIVEKIT_ENABLE_DATA_PACKET: strict ? '1' : '0',
					LIVEKIT_ENABLE_SUBSCRIPTION_CYCLE: strict ? '1' : '0',
					LIVEKIT_ENABLE_CAMERA: strict ? '1' : '0',
					LIVEKIT_ENABLE_MICROPHONE: '1',
					LIVEKIT_SCREEN_CODECS: strict ? strictCodecMatrix.join(',') : 'vp8',
					LIVEKIT_EXPECT_SCREEN_CODECS: strict ? strictCodecMatrix.join(',') : 'vp8',
					LIVEKIT_HARNESS_REPORT_PATH: path.join(reportDir, 'livekit-harness.json'),
				},
				strict
					? {
							FLUXER_WEBRTC_SENDER_LIVEKIT_REQUIRED: '1',
							LIVEKIT_REQUIRED: '1',
							LIVEKIT_HARNESS_STRICT: '1',
							LIVEKIT_ENABLE_SECOND_PUBLISHER: '1',
							LIVEKIT_ENABLE_MICROPHONE: '1',
							LIVEKIT_ENABLE_SCREEN_AUDIO: '1',
							LIVEKIT_ENABLE_CAMERA: '1',
							LIVEKIT_ENABLE_DATA_PACKET: '1',
							LIVEKIT_ENABLE_SUBSCRIPTION_CYCLE: '1',
						}
					: {},
			),
		});
	}

	commands.push(...platformNativeCommands());

	if (electronBuildEnabled) {
		commands.push({name: 'electron-build', command: `${packageManager} build`, category: 'electron'});
	}

	return commands;
}

function makeGate(name, status, details = {}) {
	return {
		name,
		status,
		...details,
	};
}

function modeGate() {
	return makeGate(
		'mode-valid',
		validModes.has(mode) ? 'pass' : 'fail',
		validModes.has(mode) ? {mode} : {mode, issues: [`FLUXER_NATIVE_MEDIA_MODE must be smoke or strict; got ${mode}`]},
	);
}

async function documentationGate() {
	const docPath = path.join(repoDir, 'native', 'NATIVE_MEDIA_INTEGRATION.md');
	const packagePath = path.join(repoDir, 'package.json');
	const missing = [];
	try {
		const [doc, packageJson] = await Promise.all([readFile(docPath, 'utf8'), readFile(packagePath, 'utf8')]);
		const packageScripts = JSON.parse(packageJson).scripts ?? {};
		const requiredSnippets = [
			'FLUXER_NATIVE_MEDIA_MODE=strict',
			'FLUXER_WEBRTC_SENDER_LIVEKIT_REQUIRED=1',
			'LIVEKIT_URL',
			'LIVEKIT_API_KEY',
			'LIVEKIT_API_SECRET',
			'FLUXER_NATIVE_MEDIA_REPORT_DIR',
			'test:native-media:strict',
			'fail-hard',
			'macOS host',
			'Windows 11 Parallels',
			'Linux Parallels',
		];
		for (const snippet of requiredSnippets) {
			if (!doc.includes(snippet)) missing.push(`native/NATIVE_MEDIA_INTEGRATION.md missing ${snippet}`);
		}
		if (packageScripts['test:native-media'] !== 'node scripts/native-media-integration.mjs') {
			missing.push('package.json script test:native-media does not point at native-media-integration.mjs');
		}
		if (packageScripts['test:native-media:strict'] !== 'node scripts/native-media-integration.mjs --strict') {
			missing.push('package.json script test:native-media:strict does not run strict native media integration');
		}
	} catch (error) {
		missing.push(`documentation/package script validation failed: ${error.message}`);
	}
	return makeGate('documentation-current', missing.length === 0 ? 'pass' : strict ? 'fail' : 'warn', {missing});
}

function strictPrerequisiteGates() {
	if (!strict) {
		return [makeGate('strict-prerequisites', 'skip', {reason: 'smoke mode'})];
	}

	const gates = [];
	const liveKitIssues = [];
	if (envExplicitFalse('FLUXER_NATIVE_MEDIA_LIVEKIT')) {
		liveKitIssues.push('FLUXER_NATIVE_MEDIA_LIVEKIT=0 is not allowed in strict mode');
	}
	for (const names of strictLiveKitCredentialGroups) {
		if (!envPresent(names)) liveKitIssues.push(`missing ${names.join(' or ')}`);
	}
	for (const flag of strictLiveKitFeatureFlags) {
		if (envExplicitFalse(flag)) liveKitIssues.push(`${flag}=0 is not allowed in strict mode`);
	}
	for (const codecEnvName of ['LIVEKIT_SCREEN_CODECS', 'LIVEKIT_EXPECT_SCREEN_CODECS']) {
		const codecs = parseCodecEnv(codecEnvName);
		if (!codecs) continue;
		for (const codec of strictCodecMatrix) {
			if (!codecListIncludes(codecs, codec)) {
				liveKitIssues.push(`${codecEnvName} must include ${codec} in strict mode`);
			}
		}
	}
	gates.push(makeGate('livekit-required', liveKitIssues.length === 0 ? 'pass' : 'fail', {issues: liveKitIssues}));

	const electronIssues = [];
	if (envExplicitFalse('FLUXER_NATIVE_MEDIA_ELECTRON_BUILD')) {
		electronIssues.push('FLUXER_NATIVE_MEDIA_ELECTRON_BUILD=0 is not allowed in strict mode');
	}
	gates.push(
		makeGate('electron-build-required', electronIssues.length === 0 ? 'pass' : 'fail', {issues: electronIssues}),
	);

	const platformCommands = platformNativeCommands();
	gates.push(
		makeGate(
			'platform-native-required',
			platformCommands.length > 0 ? 'pass' : 'fail',
			platformCommands.length > 0
				? {platform: process.platform, commands: platformCommands.map((command) => command.name)}
				: {
						platform: process.platform,
						issues: [`strict mode is only defined for linux, darwin, and win32; got ${process.platform}`],
					},
		),
	);

	gates.push(makeGate('summary-report-required', 'pass', {summaryPath: path.join(reportDir, 'summary.json')}));
	return gates;
}

function commandPlanGate(commands) {
	if (!strict) return makeGate('strict-command-plan', 'skip', {reason: 'smoke mode'});

	const names = new Set(commands.map((command) => command.name));
	const missing = [];
	if (!names.has('webrtc-sender-livekit-harness')) missing.push('webrtc-sender-livekit-harness');
	if (!names.has('electron-build')) missing.push('electron-build');
	for (const command of platformNativeCommands()) {
		if (!names.has(command.name)) missing.push(command.name);
	}
	return makeGate('strict-command-plan', missing.length === 0 ? 'pass' : 'fail', {missing});
}

async function evaluateGates(commands) {
	return [modeGate(), await documentationGate(), ...strictPrerequisiteGates(), commandPlanGate(commands)];
}

function runCommand(step) {
	const startedAtMs = Date.now();
	return new Promise((resolve) => {
		console.log(`[native-media] ${step.name}: ${step.command}`);
		const child = spawn(step.command, {
			cwd: step.cwd ? path.join(repoDir, step.cwd) : repoDir,
			env: step.env ?? withDefaultEnv({}),
			shell: true,
			stdio: ['ignore', 'pipe', 'pipe'],
			windowsHide: true,
		});
		let stdoutTail = '';
		let stderrTail = '';
		const appendTail = (current, chunk) => `${current}${chunk}`.slice(-12_000);
		child.stdout.on('data', (chunk) => {
			const text = chunk.toString();
			stdoutTail = appendTail(stdoutTail, text);
			process.stdout.write(text);
		});
		child.stderr.on('data', (chunk) => {
			const text = chunk.toString();
			stderrTail = appendTail(stderrTail, text);
			process.stderr.write(text);
		});
		child.on('close', (code, signal) => {
			const endedAtMs = Date.now();
			resolve({
				name: step.name,
				command: step.command,
				category: step.category ?? 'other',
				status: code === 0 ? 'pass' : 'fail',
				code,
				signal,
				startedAt: new Date(startedAtMs).toISOString(),
				endedAt: new Date(endedAtMs).toISOString(),
				durationMs: endedAtMs - startedAtMs,
				stdoutTail,
				stderrTail,
			});
		});
	});
}

async function writeSummary({startedAtMs, commands, results, gates, failedGate}) {
	const failedCommand = results.find((result) => result.status === 'fail');
	const report = {
		status: failedGate || failedCommand ? 'fail' : 'pass',
		mode,
		strict,
		platform: process.platform,
		arch: process.arch,
		hostname: os.hostname(),
		startedAt: new Date(startedAtMs).toISOString(),
		endedAt: new Date().toISOString(),
		reportDir,
		gates,
		plannedCommands: commands.map((command) => ({
			name: command.name,
			command: command.command,
			category: command.category ?? 'other',
			cwd: command.cwd ?? '.',
		})),
		commands: results,
	};
	const summaryPath = path.join(reportDir, 'summary.json');
	await writeFile(summaryPath, `${JSON.stringify(report, null, 2)}\n`, 'utf8');
	console.log(`[native-media] summary: ${summaryPath}`);
	return report.status === 'pass' ? 0 : 1;
}

async function main() {
	await mkdir(reportDir, {recursive: true});
	const startedAtMs = Date.now();
	const commands = commandPlan();
	const gates = await evaluateGates(commands);
	const failedGate = gates.find((gate) => gate.status === 'fail');
	const results = [];
	if (failedGate) {
		console.error(`[native-media] FAIL: ${failedGate.name}`);
		return writeSummary({startedAtMs, commands, results, gates, failedGate});
	}
	for (const command of commands) {
		const result = await runCommand(command);
		results.push(result);
		if (result.status !== 'pass') {
			break;
		}
	}
	return writeSummary({startedAtMs, commands, results, gates});
}

process.exitCode = await main();
