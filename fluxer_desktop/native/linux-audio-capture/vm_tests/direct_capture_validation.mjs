// SPDX-License-Identifier: AGPL-3.0-or-later

import {execFileSync} from 'node:child_process';
import {copyFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {setTimeout as sleep} from 'node:timers/promises';

const require = createRequire(import.meta.url);
const work = process.env.FLX_WORK;
const addonNode = `${work}/flx_direct_addon.node`;
copyFileSync(process.env.FLX_ADDON_SO, addonNode);
const addon = require(addonNode);

const results = [];
function record(name, ok, detail) {
	results.push({name, ok});
	console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? `  -- ${detail}` : ''}`);
}
function pwDump() {
	return JSON.parse(execFileSync('pw-dump', {encoding: 'utf8', maxBuffer: 64e6}));
}
function directSinkNode(dump) {
	return dump.find(
		(o) => o.type === 'PipeWire:Interface:Node' && /^fluxer-direct-capture-/.test(o.info?.props?.['node.name'] || ''),
	);
}
async function waitFor(p, ms, step = 150) {
	const end = Date.now() + ms;
	let last;
	while (Date.now() < end) {
		last = p();
		if (last) return last;
		await sleep(step);
	}
	return last;
}

function rms(samples) {
	if (samples.length === 0) return 0;
	let sum = 0;
	for (const s of samples) sum += s * s;
	return Math.sqrt(sum / samples.length);
}

async function main() {
	const dc = new addon.DirectAudioCapture();
	let lifecycle = null;
	dc.setLifecycleCallback((...args) => {
		const flat = args.length === 1 && Array.isArray(args[0]) ? args[0] : args;
		lifecycle = {kind: flat[0], msg: flat[1]};
	});

	const started = dc.start({include: [{'application.name': 'Music Player Demo'}]});
	record('DirectAudioCapture.start(include rule) accepted', started === true);

	const sinkUp = await waitFor(() => {
		const d = pwDump();
		return directSinkNode(d) ? d : null;
	}, 8000);
	record(
		'hidden private sink fluxer-direct-capture-* created',
		!!sinkUp,
		sinkUp ? directSinkNode(sinkUp).info.props['node.name'] : 'timeout',
	);

	if (sinkUp) {
		const sinkProps = directSinkNode(sinkUp).info.props;
		record(
			'private sink node.hidden=true (not user-visible)',
			String(sinkProps['node.hidden']) === 'true',
			`node.hidden=${sinkProps['node.hidden']}`,
		);
		record('private sink media.class=Audio/Sink', sinkProps['media.class'] === 'Audio/Sink', sinkProps['media.class']);
	}

	let maxRms = 0;
	let frames = 0;
	let totalSamples = 0;
	const captureDeadline = Date.now() + 6000;
	while (Date.now() < captureDeadline) {
		const f = dc.read();
		if (f) {
			const samples = new Float32Array(f.samples);
			frames += 1;
			totalSamples += samples.length;
			maxRms = Math.max(maxRms, rms(samples));
			if (maxRms > 0.01 && frames > 5) break;
		}
		await sleep(20);
	}
	record('DirectAudioCapture yields real frames', frames > 0, `${frames} frames, ${totalSamples} samples`);
	record('captured audio is non-silent (real 440Hz tone tapped)', maxRms > 0.01, `peak rms=${maxRms.toFixed(4)}`);

	const dump = pwDump();
	const sink = directSinkNode(dump);
	const sinkId = Number(sink?.id);
	const linkSrcNodes = new Set(
		dump
			.filter((o) => o.type === 'PipeWire:Interface:Link' && Number(o.info?.props?.['link.input.node']) === sinkId)
			.map((o) => Number(o.info?.props?.['link.output.node'])),
	);
	const fluxerStreamIds = dump
		.filter((o) => o.type === 'PipeWire:Interface:Node' && o.info?.props?.['application.name'] === 'Fluxer')
		.map((o) => Number(o.id));
	record(
		'Fluxer-named app excluded from per-process capture',
		fluxerStreamIds.every((id) => !linkSrcNodes.has(id)),
		`fluxer=${fluxerStreamIds} linkedSrc=${[...linkSrcNodes]}`,
	);

	const musicIds = dump
		.filter((o) => o.type === 'PipeWire:Interface:Node' && o.info?.props?.['application.name'] === 'Music Player Demo')
		.map((o) => Number(o.id));
	record(
		'targeted app IS linked to the private sink',
		musicIds.some((id) => linkSrcNodes.has(id)),
		`music=${musicIds} linkedSrc=${[...linkSrcNodes]}`,
	);

	dc.stop();
	await sleep(600);
	record('stop() emits closed-clean lifecycle', lifecycle?.kind === 'closed-clean', JSON.stringify(lifecycle));
	const afterStop = pwDump();
	const sinkAfter = directSinkNode(afterStop);
	const residualLinks = sinkAfter
		? afterStop.filter(
				(o) =>
					o.type === 'PipeWire:Interface:Link' && Number(o.info?.props?.['link.input.node']) === Number(sinkAfter.id),
			).length
		: 0;
	record('stop() removes capture links', residualLinks === 0, `residual=${residualLinks}`);

	const failed = results.filter((r) => !r.ok).length;
	console.log(`\n=== direct: ${results.length - failed}/${results.length} checks passed ===`);
	process.exit(failed === 0 ? 0 : 1);
}

main().catch((e) => {
	console.error('HARNESS ERROR:', e?.stack || e);
	process.exit(2);
});
