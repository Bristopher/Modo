// SPDX-License-Identifier: AGPL-3.0-or-later

import http from 'node:http';
import type {AddressInfo} from 'node:net';
import {type FakeNcmecFailure, FakeNcmecServer} from './FakeNcmecServer';

const host = process.env.FAKE_NCMEC_HOST ?? '127.0.0.1';
const port = readPort('FAKE_NCMEC_PORT', 9090);
const controlHost = process.env.FAKE_NCMEC_CONTROL_HOST ?? '127.0.0.1';
const controlPort = readPort('FAKE_NCMEC_CONTROL_PORT', port + 1);
const publicHost = process.env.FAKE_NCMEC_PUBLIC_HOST ?? (host === '0.0.0.0' ? '<reachable-host-or-ip>' : host);
const username = process.env.FAKE_NCMEC_USERNAME ?? 'usr123';
const password = process.env.FAKE_NCMEC_PASSWORD ?? 'pswd123';

if (host === controlHost && port === controlPort) {
	throw new Error('FAKE_NCMEC_CONTROL_PORT must differ from FAKE_NCMEC_PORT when both servers bind the same host');
}

const fake = new FakeNcmecServer({username, password});

function log(...parts: Array<unknown>): void {
	const ts = new Date().toISOString();
	console.log(`[fake-ncmec ${ts}]`, ...parts);
}

await fake.start({
	host,
	port,
	onRequest: (entry) => {
		log(
			`${entry.remoteAddress ?? '?'} ${entry.method} ${entry.path} -> ${entry.statusCode} ${entry.durationMs}ms (${entry.requestId})`,
		);
	},
});

const controlServer = http.createServer(async (req, res) => {
	const url = new URL(req.url ?? '/', `http://${req.headers.host ?? 'localhost'}`);
	log(`control ${req.method} ${url.pathname}`);
	if (req.method === 'GET' && url.pathname === '/_state') {
		res.writeHead(200, {'content-type': 'application/json'});
		res.end(JSON.stringify(fake.getReports(), null, 2));
		return;
	}
	if (req.method === 'GET' && url.pathname.startsWith('/_report/')) {
		const reportId = decodeURIComponent(url.pathname.slice('/_report/'.length));
		const report = fake.getReport(reportId);
		if (!report) {
			res.writeHead(404, {'content-type': 'application/json'});
			res.end(JSON.stringify({error: `Unknown report ${reportId}`}));
			return;
		}
		res.writeHead(200, {'content-type': 'application/json'});
		res.end(JSON.stringify(report, null, 2));
		return;
	}
	if (req.method === 'POST' && url.pathname === '/_reset') {
		fake.reset();
		res.writeHead(204);
		res.end();
		return;
	}
	if (req.method === 'POST' && url.pathname === '/_fail') {
		const mode = (url.searchParams.get('mode') ?? 'none') as FakeNcmecFailure;
		if (!VALID_FAILURE_MODES.has(mode)) {
			res.writeHead(400, {'content-type': 'application/json'});
			res.end(JSON.stringify({error: `Invalid failure mode ${mode}`}));
			return;
		}
		fake.setFailure(mode);
		res.writeHead(200, {'content-type': 'application/json'});
		res.end(JSON.stringify({failure: mode}));
		return;
	}
	res.writeHead(404);
	res.end('Not found. Try GET /_state, POST /_reset, POST /_fail?mode=...');
});

await new Promise<void>((resolve) => controlServer.listen(controlPort, controlHost, () => resolve()));

const address = controlServer.address() as AddressInfo;

log(`NCMEC-compatible fake server ready`);

log(`  NCMEC bind       → http://${fake.host}:${fake.port}/ispws`);

log(`  NCMEC base URL   → http://${publicHost}:${fake.port}/ispws`);

log(`  control endpoint → http://${address.address}:${address.port}/_state`);

log(`  credentials      → ${username}:${password}`);

if (publicHost.startsWith('<')) {
	log('  set FAKE_NCMEC_PUBLIC_HOST to the hostname or IP your prod API can reach');
}

log('  point prod Config.ncmec.baseUrl at the NCMEC base URL above');

function shutdown(signal: string): void {
	log(`received ${signal}, shutting down`);
	Promise.allSettled([fake.stop(), new Promise<void>((resolve) => controlServer.close(() => resolve()))]).then(() =>
		process.exit(0),
	);
}

process.on('SIGINT', () => shutdown('SIGINT'));

process.on('SIGTERM', () => shutdown('SIGTERM'));

const VALID_FAILURE_MODES = new Set<FakeNcmecFailure>([
	'none',
	'submit',
	'upload',
	'fileinfo',
	'finish',
	'retract',
	'unauthorized',
]);

function readPort(name: string, fallback: number): number {
	const raw = process.env[name];
	if (!raw) return fallback;
	const value = Number.parseInt(raw, 10);
	if (!Number.isInteger(value) || value < 0 || value > 65535) {
		throw new Error(`${name} must be an integer between 0 and 65535`);
	}
	return value;
}
