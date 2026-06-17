// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';
import fs from 'node:fs';
import http from 'node:http';
import https from 'node:https';
import path from 'node:path';
import {createChildLogger} from '@electron/common/Logger';
import {app, nativeImage} from 'electron';

const logger = createChildLogger('NotificationIcon');
const NOTIFICATION_ICON_DOWNLOAD_TIMEOUT_MS = 10000;
const NOTIFICATION_ICON_MAX_BYTES = 3 * 1024 * 1024;
const NOTIFICATION_ICON_CACHE_MAX_FILES = 512;

type ResolvedNotificationIcon = NonNullable<Electron.NotificationConstructorOptions['icon']>;

interface DownloadOptions {
	maxBytes: number;
	timeoutMs: number;
	redirectsRemaining: number;
}

function isHttpUrl(value: string): boolean {
	return value.startsWith('http://') || value.startsWith('https://');
}

function describeIconSource(source: string): string {
	if (source.startsWith('data:')) {
		return 'data-url';
	}
	try {
		const url = new URL(source);
		return `${url.protocol}//${url.host}${url.pathname}`;
	} catch {
		return path.basename(source) || 'local-path';
	}
}

function getNotificationIconCacheDir(): string {
	return path.join(app.getPath('userData'), 'notification-icons');
}

function getErrorCode(error: unknown): string | undefined {
	if (typeof error !== 'object' || error === null || !('code' in error)) {
		return undefined;
	}
	const code = (
		error as {
			code?: unknown;
		}
	).code;
	return typeof code === 'string' ? code : undefined;
}

async function touchFile(filePath: string): Promise<void> {
	const now = new Date();
	await fs.promises.utimes(filePath, now, now).catch(() => {});
}

let trimPromise: Promise<void> | null = null;

function scheduleNotificationIconCacheTrim(cacheDir: string): void {
	if (trimPromise) return;
	trimPromise = trimNotificationIconCache(cacheDir)
		.catch((error) => {
			logger.warn('Failed to trim notification icon cache', {error});
		})
		.finally(() => {
			trimPromise = null;
		});
}

async function trimNotificationIconCache(cacheDir: string): Promise<void> {
	const entries = await fs.promises.readdir(cacheDir, {withFileTypes: true});
	const files = await Promise.all(
		entries
			.filter((entry) => entry.isFile() && entry.name.endsWith('.png'))
			.map(async (entry) => {
				const filePath = path.join(cacheDir, entry.name);
				const stat = await fs.promises.stat(filePath);
				return {filePath, mtimeMs: stat.mtimeMs};
			}),
	);
	if (files.length <= NOTIFICATION_ICON_CACHE_MAX_FILES) {
		return;
	}
	files.sort((a, b) => a.mtimeMs - b.mtimeMs);
	const deleteCount = files.length - NOTIFICATION_ICON_CACHE_MAX_FILES;
	await Promise.all(files.slice(0, deleteCount).map(({filePath}) => fs.promises.rm(filePath, {force: true})));
}

async function cacheNotificationIcon(source: string, image: Electron.NativeImage): Promise<string> {
	const cacheDir = getNotificationIconCacheDir();
	await fs.promises.mkdir(cacheDir, {recursive: true});
	const cacheKey = crypto.createHash('sha256').update(source).digest('hex');
	const filePath = path.join(cacheDir, `${cacheKey}.png`);
	try {
		await fs.promises.access(filePath, fs.constants.R_OK);
		await touchFile(filePath);
		return filePath;
	} catch {}
	const tmpPath = path.join(cacheDir, `${cacheKey}.${process.pid}.${Date.now()}.tmp`);
	await fs.promises.writeFile(tmpPath, image.toPNG(), {mode: 0o600});
	try {
		await fs.promises.rename(tmpPath, filePath);
	} catch (error) {
		await fs.promises.rm(tmpPath, {force: true});
		if (getErrorCode(error) === 'EEXIST') {
			await touchFile(filePath);
			return filePath;
		}
		throw error;
	}
	scheduleNotificationIconCacheTrim(cacheDir);
	return filePath;
}

function decodeNotificationIcon(source: string, buffer: Buffer): Electron.NativeImage | null {
	const image = nativeImage.createFromBuffer(buffer);
	if (image.isEmpty()) {
		logger.warn('Notification icon could not be decoded as PNG/JPEG', {
			source: describeIconSource(source),
			bytes: buffer.length,
		});
		return null;
	}
	return image;
}

async function resolveDecodedNotificationIcon(
	source: string,
	image: Electron.NativeImage,
): Promise<ResolvedNotificationIcon> {
	if (process.platform === 'win32') {
		return cacheNotificationIcon(source, image);
	}
	return image;
}

export async function resolveNotificationIcon(source: string): Promise<ResolvedNotificationIcon | null> {
	if (!source) {
		return null;
	}
	if (isHttpUrl(source)) {
		const buffer = await downloadToBuffer(source, {
			maxBytes: NOTIFICATION_ICON_MAX_BYTES,
			timeoutMs: NOTIFICATION_ICON_DOWNLOAD_TIMEOUT_MS,
			redirectsRemaining: 5,
		});
		const image = decodeNotificationIcon(source, buffer);
		return image ? resolveDecodedNotificationIcon(source, image) : null;
	}
	if (source.startsWith('data:')) {
		const image = nativeImage.createFromDataURL(source);
		if (image.isEmpty()) {
			logger.warn('Notification icon data URL could not be decoded as PNG/JPEG');
			return null;
		}
		return resolveDecodedNotificationIcon(source, image);
	}
	return source;
}

function downloadToBuffer(url: string, options: DownloadOptions): Promise<Buffer> {
	return new Promise((resolve, reject) => {
		const parsedUrl = new URL(url);
		const protocol = parsedUrl.protocol === 'https:' ? https : http;
		let settled = false;
		const finish = (error: Error | null, buffer?: Buffer): void => {
			if (settled) return;
			settled = true;
			if (error) {
				reject(error);
			} else {
				resolve(buffer ?? Buffer.alloc(0));
			}
		};
		const request = protocol.get(parsedUrl, (response) => {
			const statusCode = response.statusCode ?? 0;
			if ([301, 302, 303, 307, 308].includes(statusCode)) {
				const location = response.headers.location;
				response.resume();
				if (!location) {
					finish(new Error(`Notification icon redirect missing Location header (${statusCode})`));
					return;
				}
				if (options.redirectsRemaining <= 0) {
					finish(new Error('Notification icon download exceeded redirect limit'));
					return;
				}
				const redirectUrl = new URL(location, parsedUrl).toString();
				downloadToBuffer(redirectUrl, {...options, redirectsRemaining: options.redirectsRemaining - 1})
					.then(resolve)
					.catch(reject);
				settled = true;
				return;
			}
			if (statusCode !== 200) {
				response.resume();
				finish(new Error(`Notification icon download failed with HTTP ${statusCode}`));
				return;
			}
			const contentLengthRaw = response.headers['content-length'];
			const contentLength = Array.isArray(contentLengthRaw) ? Number(contentLengthRaw[0]) : Number(contentLengthRaw);
			if (Number.isFinite(contentLength) && contentLength > options.maxBytes) {
				response.resume();
				finish(new Error(`Notification icon exceeds ${options.maxBytes} bytes`));
				return;
			}
			const chunks: Array<Buffer> = [];
			let totalBytes = 0;
			response.on('data', (chunk: Buffer) => {
				totalBytes += chunk.length;
				if (totalBytes > options.maxBytes) {
					request.destroy(new Error(`Notification icon exceeds ${options.maxBytes} bytes`));
					return;
				}
				chunks.push(chunk);
			});
			response.on('end', () => finish(null, Buffer.concat(chunks)));
			response.on('error', finish);
		});
		request.setTimeout(options.timeoutMs, () => {
			request.destroy(new Error(`Notification icon download timed out after ${options.timeoutMs}ms`));
		});
		request.on('error', finish);
	});
}
