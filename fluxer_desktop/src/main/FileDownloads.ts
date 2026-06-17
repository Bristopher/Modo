// SPDX-License-Identifier: AGPL-3.0-or-later

import fs from 'node:fs';
import http from 'node:http';
import https from 'node:https';
import {pipeline} from 'node:stream/promises';

const MAX_DOWNLOAD_REDIRECTS = 5;
const DOWNLOAD_TIMEOUT_MS = 30000;

function parseHttpUrl(url: string): URL {
	const parsed = new URL(url);
	if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
		throw new Error('Download URL must use http or https');
	}
	return parsed;
}

function requestUrl(url: URL): Promise<http.IncomingMessage> {
	const transport = url.protocol === 'https:' ? https : http;
	return new Promise((resolve, reject) => {
		const request = transport.get(url, resolve);
		request.setTimeout(DOWNLOAD_TIMEOUT_MS, () => {
			request.destroy(new Error('Download timed out'));
		});
		request.on('error', reject);
	});
}

async function removePartialDownload(destPath: string): Promise<void> {
	await fs.promises.unlink(destPath).catch(() => {});
}

async function downloadFileWithRedirects(url: URL, destPath: string, redirects: number): Promise<void> {
	const response = await requestUrl(url);
	const statusCode = response.statusCode ?? 0;
	if (statusCode >= 300 && statusCode < 400) {
		response.resume();
		const location = response.headers.location;
		if (!location) {
			throw new Error(`HTTP ${statusCode} redirect missing Location header`);
		}
		if (redirects >= MAX_DOWNLOAD_REDIRECTS) {
			throw new Error('Too many download redirects');
		}
		const nextUrl = parseHttpUrl(new URL(location, url).toString());
		await downloadFileWithRedirects(nextUrl, destPath, redirects + 1);
		return;
	}
	if (statusCode === 204 || statusCode === 205) {
		response.resume();
		await fs.promises.writeFile(destPath, new Uint8Array());
		return;
	}
	if (statusCode < 200 || statusCode >= 300) {
		response.resume();
		throw new Error(`HTTP ${statusCode}`);
	}
	try {
		await pipeline(response, fs.createWriteStream(destPath));
	} catch (error) {
		await removePartialDownload(destPath);
		throw error;
	}
}

export async function downloadFile(url: string, destPath: string): Promise<void> {
	await removePartialDownload(destPath);
	await downloadFileWithRedirects(parseHttpUrl(url), destPath, 0);
}
