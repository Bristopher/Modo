// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';
import http from 'node:http';
import type {AddressInfo} from 'node:net';
import {Readable} from 'node:stream';
import {XMLParser, XMLValidator} from 'fast-xml-parser';

export type FakeNcmecFailure = 'none' | 'submit' | 'upload' | 'fileinfo' | 'finish' | 'retract' | 'unauthorized';

export interface FakeNcmecFileRecord {
	fileId: string;
	filename: string;
	size: number;
	md5: string;
	submittedDetails: boolean;
	detailsXml: string | null;
}

export interface FakeNcmecReportRecord {
	reportId: string;
	reportXml: string;
	files: Array<FakeNcmecFileRecord>;
	finished: boolean;
	retracted: boolean;
	authUsername: string;
}

interface FakeNcmecServerOptions {
	username: string;
	password: string;
}

interface FakeNcmecServerStartOptions {
	host?: string;
	port?: number;
	onRequest?: (entry: FakeNcmecRequestLogEntry) => void;
}

interface FakeNcmecRequestLogEntry {
	method: string;
	path: string;
	statusCode: number;
	requestId: string;
	durationMs: number;
	remoteAddress: string | null;
}

const xmlParser = new XMLParser({ignoreAttributes: false, parseTagValue: false, trimValues: true});

export class FakeNcmecServer {
	private server: http.Server | null = null;
	private listenPort = 0;
	private listenHost = '127.0.0.1';
	private onRequest: ((entry: FakeNcmecRequestLogEntry) => void) | null = null;
	private reportCounter = 1000000;
	private fileCounter = 1;
	private requestCounter = 1;
	private readonly reports = new Map<string, FakeNcmecReportRecord>();
	private failureMode: FakeNcmecFailure = 'none';

	constructor(private readonly options: FakeNcmecServerOptions) {}

	async start(startOptions: FakeNcmecServerStartOptions = {}): Promise<void> {
		if (this.server) {
			throw new Error('FakeNcmecServer already started');
		}
		this.listenHost = startOptions.host ?? '127.0.0.1';
		this.onRequest = startOptions.onRequest ?? null;
		this.server = http.createServer((req, res) => this.handle(req, res));
		await new Promise<void>((resolve) => this.server!.listen(startOptions.port ?? 0, this.listenHost, () => resolve()));
		this.listenPort = (this.server!.address() as AddressInfo).port;
	}

	async stop(): Promise<void> {
		if (!this.server) return;
		await new Promise<void>((resolve, reject) => {
			this.server!.close((error) => (error ? reject(error) : resolve()));
		});
		this.server = null;
		this.listenPort = 0;
		this.onRequest = null;
	}

	get baseUrl(): string {
		if (!this.server) throw new Error('FakeNcmecServer not started');
		return `http://${this.listenHost}:${this.listenPort}/ispws`;
	}

	get host(): string {
		if (!this.server) throw new Error('FakeNcmecServer not started');
		return this.listenHost;
	}

	get port(): number {
		if (!this.server) throw new Error('FakeNcmecServer not started');
		return this.listenPort;
	}

	reset(): void {
		this.reports.clear();
		this.reportCounter = 1000000;
		this.fileCounter = 1;
		this.requestCounter = 1;
		this.failureMode = 'none';
	}

	setFailure(mode: FakeNcmecFailure): void {
		this.failureMode = mode;
	}

	getReports(): Array<FakeNcmecReportRecord> {
		return [...this.reports.values()];
	}

	getReport(reportId: string): FakeNcmecReportRecord | null {
		return this.reports.get(reportId) ?? null;
	}

	private async handle(req: http.IncomingMessage, res: http.ServerResponse): Promise<void> {
		try {
			const url = new URL(req.url ?? '/', 'http://127.0.0.1');
			const pathname = url.pathname.replace(/^\/ispws/, '') || '/';
			const requestId = `req-${this.requestCounter++}`;
			const startedAt = Date.now();
			res.setHeader('Request-ID', requestId);
			res.on('finish', () => {
				this.onRequest?.({
					method: req.method ?? 'GET',
					path: pathname,
					statusCode: res.statusCode,
					requestId,
					durationMs: Date.now() - startedAt,
					remoteAddress: req.socket.remoteAddress ?? null,
				});
			});
			if (this.failureMode === 'unauthorized') {
				return this.sendStatus(res, 401, '');
			}
			if (!this.checkBasicAuth(req)) {
				return this.sendStatus(res, 401, '');
			}
			const injectedHeader = (req.headers['x-test-fail'] ?? '').toString() as FakeNcmecFailure;
			const effectiveFailure: FakeNcmecFailure = injectedHeader || this.failureMode;
			if (req.method === 'GET' && pathname === '/status') {
				const remoteIp = req.socket.remoteAddress ?? 'unknown';
				return this.sendXml(
					res,
					200,
					ok(
						`<responseDescription>Remote User : ${this.options.username}, Remote Ip : ${remoteIp}</responseDescription>`,
					),
				);
			}
			if (req.method === 'GET' && pathname === '/xsd') {
				return this.sendText(res, 200, '<schema></schema>');
			}
			if (req.method !== 'POST') {
				return this.sendStatus(res, 405, '');
			}
			const request = await this.toWebRequest(req, url);
			switch (pathname) {
				case '/submit':
					return this.handleSubmit(request, res, effectiveFailure);
				case '/upload':
					return this.handleUpload(request, res, effectiveFailure);
				case '/fileinfo':
					return this.handleFileInfo(request, res, effectiveFailure);
				case '/finish':
					return this.handleFinish(request, res, effectiveFailure);
				case '/retract':
					return this.handleRetract(request, res, effectiveFailure);
				default:
					return this.sendStatus(res, 404, '');
			}
		} catch (error) {
			this.sendStatus(res, 500, String(error));
		}
	}

	private async handleSubmit(request: Request, res: http.ServerResponse, failure: FakeNcmecFailure): Promise<void> {
		if (failure === 'submit') {
			return this.sendXml(res, 200, errorXml(1100, 'Save failed'));
		}
		if (!isXmlContentType(request)) {
			return this.sendXml(res, 200, errorXml(4000, 'Invalid request'));
		}
		const body = await request.text();
		if (XMLValidator.validate(body) !== true) {
			return this.sendXml(res, 200, errorXml(4110, 'Malformed XML submittal'));
		}
		const report = parseReportDocument(body);
		if (!hasRequiredReportFields(report)) {
			return this.sendXml(res, 200, errorXml(4100, 'Validation failed'));
		}
		const reportId = String(this.reportCounter++);
		this.reports.set(reportId, {
			reportId,
			reportXml: body,
			files: [],
			finished: false,
			retracted: false,
			authUsername: this.options.username,
		});
		return this.sendXml(res, 200, ok(`<reportId>${reportId}</reportId>`));
	}

	private async handleUpload(request: Request, res: http.ServerResponse, failure: FakeNcmecFailure): Promise<void> {
		if (failure === 'upload') {
			return this.sendXml(res, 200, errorXml(1111, 'File upload failed'));
		}
		const form = await request.formData();
		const reportId = String(form.get('id') ?? '');
		const file = form.get('file');
		const record = this.reports.get(reportId);
		if (!reportId || !record) {
			return this.sendXml(res, 200, errorXml(5001, 'Report does not exist'));
		}
		if (record.retracted) {
			return this.sendXml(res, 200, errorXml(5101, 'Report already retracted'));
		}
		if (record.finished) {
			return this.sendXml(res, 200, errorXml(5102, 'Report already finished'));
		}
		if (!(file instanceof Blob)) {
			return this.sendXml(res, 200, errorXml(4200, 'Malformed file submittal'));
		}
		const buffer = Buffer.from(await file.arrayBuffer());
		const md5 = crypto.createHash('md5').update(buffer).digest('hex');
		const fileId = `file-${this.fileCounter++}`;
		record.files.push({
			fileId,
			filename: (file as File).name || 'upload.bin',
			size: buffer.byteLength,
			md5,
			submittedDetails: false,
			detailsXml: null,
		});
		return this.sendXml(res, 200, ok(`<reportId>${reportId}</reportId><fileId>${fileId}</fileId><hash>${md5}</hash>`));
	}

	private async handleFileInfo(request: Request, res: http.ServerResponse, failure: FakeNcmecFailure): Promise<void> {
		if (failure === 'fileinfo') {
			return this.sendXml(res, 200, errorXml(1300, 'Update failed'));
		}
		if (!isXmlContentType(request)) {
			return this.sendXml(res, 200, errorXml(4000, 'Invalid request'));
		}
		const body = await request.text();
		if (XMLValidator.validate(body) !== true) {
			return this.sendXml(res, 200, errorXml(4110, 'Malformed XML submittal'));
		}
		const fileDetails = parseFileDetailsDocument(body);
		if (!hasFileDetailsFields(fileDetails)) {
			return this.sendXml(res, 200, errorXml(4100, 'Validation failed'));
		}
		const reportId = nonBlankString(fileDetails?.reportId);
		const fileId = nonBlankString(fileDetails?.fileId);
		const record = reportId ? this.reports.get(reportId) : null;
		if (!record) {
			return this.sendXml(res, 200, errorXml(5001, 'Report does not exist'));
		}
		if (record.retracted) {
			return this.sendXml(res, 200, errorXml(5101, 'Report already retracted'));
		}
		if (record.finished) {
			return this.sendXml(res, 200, errorXml(5102, 'Report already finished'));
		}
		const file = record.files.find((f) => f.fileId === fileId);
		if (!file) {
			return this.sendXml(res, 200, errorXml(5002, 'File does not exist'));
		}
		file.submittedDetails = true;
		file.detailsXml = body;
		return this.sendXml(res, 200, ok(`<reportId>${reportId}</reportId>`));
	}

	private async handleFinish(request: Request, res: http.ServerResponse, failure: FakeNcmecFailure): Promise<void> {
		if (failure === 'finish') {
			return this.sendXml(res, 200, errorXml(1300, 'Update failed'));
		}
		const form = await request.formData();
		const reportId = String(form.get('id') ?? '');
		const record = this.reports.get(reportId);
		if (!record) {
			return this.sendXml(res, 200, errorXml(5001, 'Report does not exist'));
		}
		if (record.retracted) {
			return this.sendXml(res, 200, errorXml(5101, 'Report already retracted'));
		}
		if (record.finished) {
			return this.sendXml(res, 200, errorXml(5102, 'Report already finished'));
		}
		record.finished = true;
		const files = record.files.map((f) => `    <fileId>${f.fileId}</fileId>`).join('\n');
		const body = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<reportDoneResponse>
    <responseCode>0</responseCode>
    <reportId>${reportId}</reportId>
    <files>
${files}
    </files>
</reportDoneResponse>`;
		return this.sendXml(res, 200, body);
	}

	private async handleRetract(request: Request, res: http.ServerResponse, failure: FakeNcmecFailure): Promise<void> {
		if (failure === 'retract') {
			return this.sendXml(res, 200, errorXml(1300, 'Update failed'));
		}
		const form = await request.formData();
		const reportId = String(form.get('id') ?? '');
		const record = this.reports.get(reportId);
		if (!record) {
			return this.sendXml(res, 200, errorXml(5001, 'Report does not exist'));
		}
		if (record.retracted) {
			return this.sendXml(res, 200, errorXml(5101, 'Report already retracted'));
		}
		if (record.finished) {
			return this.sendXml(res, 200, errorXml(5102, 'Report already finished'));
		}
		record.retracted = true;
		return this.sendXml(res, 200, ok(`<reportId>${reportId}</reportId>`));
	}

	private checkBasicAuth(req: http.IncomingMessage): boolean {
		const header = req.headers.authorization ?? '';
		if (!header.toLowerCase().startsWith('basic ')) return false;
		const decoded = Buffer.from(header.slice(6), 'base64').toString('utf-8');
		const idx = decoded.indexOf(':');
		if (idx === -1) return false;
		const user = decoded.slice(0, idx);
		const pass = decoded.slice(idx + 1);
		return user === this.options.username && pass === this.options.password;
	}

	private async toWebRequest(req: http.IncomingMessage, url: URL): Promise<Request> {
		const headers = new Headers();
		for (const [k, v] of Object.entries(req.headers)) {
			if (v === undefined) continue;
			if (Array.isArray(v)) headers.set(k, v.join(','));
			else headers.set(k, String(v));
		}
		const method = req.method ?? 'GET';
		const body = method === 'GET' || method === 'HEAD' ? undefined : (Readable.toWeb(req) as ReadableStream);
		return new Request(url, {method, headers, body, duplex: 'half'} as RequestInit & {
			duplex: 'half';
		});
	}

	private sendXml(res: http.ServerResponse, status: number, body: string): void {
		res.writeHead(status, {'content-type': 'application/xml; charset=utf-8'});
		res.end(body);
	}

	private sendText(res: http.ServerResponse, status: number, body: string): void {
		res.writeHead(status, {'content-type': 'text/plain; charset=utf-8'});
		res.end(body);
	}

	private sendStatus(res: http.ServerResponse, status: number, body: string): void {
		res.writeHead(status);
		res.end(body);
	}
}

function ok(extra: string): string {
	return `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<reportResponse>
    <responseCode>0</responseCode>
    <responseDescription>Success</responseDescription>
    ${extra}
</reportResponse>`;
}

function errorXml(code: number, description: string): string {
	return `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<reportResponse>
    <responseCode>${code}</responseCode>
    <responseDescription>${description}</responseDescription>
</reportResponse>`;
}

function isXmlContentType(request: Request): boolean {
	return (request.headers.get('content-type') ?? '').toLowerCase().includes('text/xml');
}

function parseReportDocument(xml: string): Record<string, unknown> | null {
	const parsed = xmlParser.parse(xml) as Record<string, unknown>;
	return objectField(parsed, 'report');
}

function parseFileDetailsDocument(xml: string): Record<string, unknown> | null {
	const parsed = xmlParser.parse(xml) as Record<string, unknown>;
	return objectField(parsed, 'fileDetails');
}

function hasRequiredReportFields(report: Record<string, unknown> | null): boolean {
	const incidentSummary = objectField(report, 'incidentSummary');
	const reporter = objectField(report, 'reporter');
	const reportingPerson = objectField(reporter, 'reportingPerson');
	return Boolean(
		nonBlankString(incidentSummary?.incidentType) &&
			nonBlankString(incidentSummary?.incidentDateTime) &&
			nonBlankString(reportingPerson?.email),
	);
}

function hasFileDetailsFields(fileDetails: Record<string, unknown> | null): boolean {
	return Boolean(nonBlankString(fileDetails?.reportId) && nonBlankString(fileDetails?.fileId));
}

function objectField(value: unknown, key: string): Record<string, unknown> | null {
	if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
	const field = (value as Record<string, unknown>)[key];
	if (!field || typeof field !== 'object' || Array.isArray(field)) return null;
	return field as Record<string, unknown>;
}

function nonBlankString(value: unknown): string | null {
	if (typeof value === 'string') {
		const trimmed = value.trim();
		return trimmed || null;
	}
	if (typeof value === 'number' || typeof value === 'bigint') {
		return String(value);
	}
	return null;
}
