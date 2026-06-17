// SPDX-License-Identifier: AGPL-3.0-or-later

import {execFileSync, spawn} from 'node:child_process';
import {copyFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {setTimeout as sleep} from 'node:timers/promises';

const require = createRequire(import.meta.url);

const addonSo = req('FLX_ADDON_SO');
const work = req('FLX_WORK');
const tone = req('FLX_TONE');
const addonNode = `${work}/flx_audio_addon.node`;
copyFileSync(addonSo, addonNode);
const addon = require(addonNode);

const SINK_NAME = 'fluxer-screen-share';
const SINK_DESC = 'Fluxer Screen Share Audio';
const results = [];
const children = [];

function req(name) {
	const v = process.env[name];
	if (!v) {
		console.error(`HARNESS ERROR: ${name} not set`);
		process.exit(2);
	}
	return v;
}
function record(name, ok, detail) {
	results.push({name, ok});
	console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? `  -- ${detail}` : ''}`);
}
function pwDump() {
	return JSON.parse(execFileSync('pw-dump', {encoding: 'utf8', maxBuffer: 64e6}));
}
function nodesByName(dump, name) {
	return dump.filter((o) => o.type === 'PipeWire:Interface:Node' && o.info?.props?.['node.name'] === name);
}
function links(dump) {
	return dump
		.filter((o) => o.type === 'PipeWire:Interface:Link')
		.map((o) => ({
			inNode: Number(o.info?.props?.['link.input.node']),
			outNode: Number(o.info?.props?.['link.output.node']),
		}));
}
function streamNodes(dump) {
	return dump.filter(
		(o) => o.type === 'PipeWire:Interface:Node' && o.info?.props?.['media.class'] === 'Stream/Output/Audio',
	);
}
async function waitFor(predicate, timeoutMs, stepMs = 150) {
	const deadline = Date.now() + timeoutMs;
	let last;
	while (Date.now() < deadline) {
		last = predicate();
		if (last) return last;
		await sleep(stepMs);
	}
	return last;
}
function killAll() {
	for (const c of children) {
		try {
			c.kill('SIGKILL');
		} catch {}
	}
}

async function main() {
	record('backend is pipewire', addon.audioBackend() === 'pipewire', addon.audioBackend());

	const descendant = spawn(
		'pw-play',
		['--target', 'test_speakers', '-P', '{ application.name = "Descendant Player" }', tone],
		{
			stdio: 'ignore',
			env: process.env,
		},
	);
	children.push(descendant);

	const speakerIdOf = (d) => nodesByName(d, 'test_speakers')[0]?.id;
	const sourcesInto = (dump) => {
		const sid = Number(speakerIdOf(dump));
		return new Set(
			links(dump)
				.filter((l) => l.inNode === sid)
				.map((l) => l.outNode),
		).size;
	};

	const pre = await waitFor(() => {
		const d = pwDump();
		return streamNodes(d).length >= 5 && sourcesInto(d) >= 5 ? d : null;
	}, 12000);
	record(
		'all 5 playback streams routed to speakers pre-capture',
		!!pre,
		pre ? `${streamNodes(pre).length} streams, ${sourcesInto(pre)} routed` : 'timeout',
	);
	const preDump = pre || pwDump();

	const speakers = nodesByName(preDump, 'test_speakers');
	record('default sink test_speakers exists', speakers.length === 1);
	const preSpeakerSources = sourcesInto(preDump);
	record(
		'every app is playing to the real speakers pre-capture',
		preSpeakerSources >= 5,
		`${preSpeakerSources} distinct app streams -> speakers`,
	);

	const bridge = new addon.AudioBridge();
	record('AudioBridge on pipewire', bridge.backend() === 'pipewire', bridge.backend());
	record(
		'apply(system rule) accepted',
		bridge.apply({onlySpeakers: true, onlyDefaultSpeakers: true, ignoreDevices: true}) === true,
	);

	const after = await waitFor(() => {
		const d = pwDump();
		if (nodesByName(d, SINK_NAME).length !== 1) return null;
		const g = bridge.routingGraph();
		return g.ownedLinks.length >= 2 ? {d, g} : null;
	}, 10000);

	if (!after) {
		record('fluxer sink + capture links established', false, 'timeout');
		await finish(bridge);
		return;
	}
	const {d: dump, g: graph} = after;

	const sink = nodesByName(dump, SINK_NAME);
	record('exactly one fluxer-screen-share node', sink.length === 1, `count=${sink.length}`);
	const sp = sink[0]?.info?.props ?? {};
	record(
		'sink node.description is "Fluxer Screen Share Audio"',
		sp['node.description'] === SINK_DESC,
		sp['node.description'],
	);
	record('sink media.class is Audio/Source/Virtual', sp['media.class'] === 'Audio/Source/Virtual', sp['media.class']);
	record('sink node.virtual=true', String(sp['node.virtual']) === 'true');
	const sinkId = Number(sink[0]?.id);

	record(
		'all owned links are passive',
		graph.ownedLinks.every((l) => l.passive === true),
		`${graph.ownedLinks.length} links`,
	);
	record(
		'all owned links terminate at the fluxer sink',
		graph.ownedLinks.every((l) => Number(l.inputNodeId) === sinkId),
	);

	const captured = new Set(graph.ownedLinks.map((l) => Number(l.outputNodeId)));
	const idsByPredicate = (pred) =>
		streamNodes(dump)
			.filter((o) => pred(o.info.props))
			.map((o) => Number(o.id));

	const normalIds = idsByPredicate(
		(p) =>
			['pw-play', 'Music Player Demo'].includes(p['application.name']) &&
			p['application.name'] !== 'Fluxer' &&
			p['application.name'] !== 'Descendant Player' &&
			!(p['node.name'] || '').startsWith('Fluxer '),
	);
	const fluxerAppIds = idsByPredicate((p) => p['application.name'] === 'Fluxer');
	const fluxerNodeIds = idsByPredicate((p) => (p['node.name'] || '').startsWith('Fluxer '));
	const descendantIds = idsByPredicate((p) => p['application.name'] === 'Descendant Player');

	record(
		'normal external apps ARE captured',
		normalIds.length >= 2 && normalIds.every((id) => captured.has(id)),
		`normal=${normalIds} captured=${[...captured]}`,
	);
	record(
		'Fluxer-named app (application.name) is EXCLUDED',
		fluxerAppIds.length >= 1 && fluxerAppIds.every((id) => !captured.has(id)),
		`fluxerApp=${fluxerAppIds}`,
	);
	record(
		'Fluxer-named app (node.name prefix) is EXCLUDED',
		fluxerNodeIds.length >= 1 && fluxerNodeIds.every((id) => !captured.has(id)),
		`fluxerNode=${fluxerNodeIds}`,
	);
	record(
		'descendant-PID player is EXCLUDED (self-process tree)',
		descendantIds.length >= 1 && descendantIds.every((id) => !captured.has(id)),
		`descendant=${descendantIds}`,
	);

	const afterSpeakerSources = sourcesInto(dump);
	record(
		'apps STILL play to real speakers during capture (tap, not move)',
		afterSpeakerSources >= preSpeakerSources,
		`before=${preSpeakerSources} after=${afterSpeakerSources} distinct app streams -> speakers`,
	);

	const meta = dump.find((o) => o.type === 'PipeWire:Interface:Metadata' && o.props?.['metadata.name'] === 'default');
	const def = meta?.metadata?.find((m) => m.key === 'default.audio.sink')?.value?.name;
	record('default audio sink unchanged (test_speakers)', def === 'test_speakers', `default=${def}`);

	await finish(bridge);
}

async function finish(bridge) {
	bridge.release();
	const cleared = await waitFor(() => (bridge.routingGraph().ownedLinks.length === 0 ? true : null), 5000);
	record(
		'release()+settle removes all owned links',
		!!cleared,
		cleared ? '0 owned links' : `still ${bridge.routingGraph().ownedLinks.length}`,
	);
	await sleep(500);
	const d = pwDump();
	const sinkId = Number(nodesByName(d, SINK_NAME)[0]?.id);
	const residual = Number.isNaN(sinkId) ? 0 : links(d).filter((l) => l.inNode === sinkId).length;
	record('no residual links into fluxer sink after release', residual === 0, `residual=${residual}`);
	killAll();
	const failed = results.filter((r) => !r.ok).length;
	console.log(`\n=== ${results.length - failed}/${results.length} checks passed ===`);
	process.exit(failed === 0 ? 0 : 1);
}

process.on('exit', killAll);
main().catch((e) => {
	killAll();
	console.error('HARNESS ERROR:', e?.stack || e);
	process.exit(2);
});
