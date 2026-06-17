// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';
import fs from 'node:fs/promises';
import path from 'node:path';
import type {
	VoiceBackgroundMediaCacheRequest,
	VoiceBackgroundMediaCacheResult,
	VoiceBackgroundMediaKind,
	VoiceBackgroundMediaReadResult,
} from '@electron/common/Types';
import {app, ipcMain} from 'electron';

const CACHE_DIR_NAME = 'voice-background-media';
const MAX_BACKGROUND_MEDIA_BYTES = 10 * 1024 * 1024;
const MAX_BACKGROUND_MEDIA_CACHE_FILES = 16;
const MAX_BACKGROUND_MEDIA_ID_LENGTH = 128;

const MEDIA_TYPES = new Map<
	string,
	{
		mimeType: string;
		extension: string;
		mediaKind: VoiceBackgroundMediaKind;
	}
>([
	['image/jpeg', {mimeType: 'image/jpeg', extension: 'jpg', mediaKind: 'static'}],
	['image/png', {mimeType: 'image/png', extension: 'png', mediaKind: 'static'}],
	['image/gif', {mimeType: 'image/gif', extension: 'gif', mediaKind: 'animated'}],
	['image/webp', {mimeType: 'image/webp', extension: 'webp', mediaKind: 'static'}],
	['video/mp4', {mimeType: 'video/mp4', extension: 'mp4', mediaKind: 'video'}],
]);

const MEDIA_TYPES_BY_EXTENSION = new Map(Array.from(MEDIA_TYPES.values()).map((value) => [value.extension, value]));

function getCacheDir(): string {
	return path.join(app.getPath('userData'), CACHE_DIR_NAME);
}

function getCacheKey(id: string): string {
	assertValidBackgroundId(id);
	return crypto.createHash('sha256').update(id).digest('hex');
}

function assertValidBackgroundId(id: string): void {
	if (id.length === 0 || id.length > MAX_BACKGROUND_MEDIA_ID_LENGTH) {
		throw new Error('Invalid voice background media id');
	}
}

function getMediaType(options: {mimeType: string; fileName?: string}): {
	mimeType: string;
	extension: string;
	mediaKind: VoiceBackgroundMediaKind;
} {
	const normalizedMimeType = options.mimeType.trim().toLowerCase();
	const mediaType = MEDIA_TYPES.get(normalizedMimeType);
	if (!mediaType) {
		const extension = path
			.extname(options.fileName ?? '')
			.slice(1)
			.toLowerCase();
		const mediaTypeByExtension = MEDIA_TYPES_BY_EXTENSION.get(extension);
		if (!mediaTypeByExtension) {
			throw new Error('Unsupported voice background media type');
		}
		return mediaTypeByExtension;
	}
	return mediaType;
}

function isAnimatedWebp(data: Buffer): boolean {
	if (data.byteLength < 21) {
		return false;
	}
	if (data.subarray(0, 4).toString('ascii') !== 'RIFF') {
		return false;
	}
	if (data.subarray(8, 12).toString('ascii') !== 'WEBP') {
		return false;
	}
	if (data.subarray(12, 16).toString('ascii') === 'VP8X') {
		return (data[20] & 0b0000_0010) !== 0;
	}
	return data.includes(Buffer.from('ANMF', 'ascii'));
}

function classifyMediaKind(
	mediaType: {mimeType: string; extension: string; mediaKind: VoiceBackgroundMediaKind},
	data: Buffer,
): VoiceBackgroundMediaKind {
	if (mediaType.extension === 'webp' && isAnimatedWebp(data)) {
		return 'animated';
	}
	return mediaType.mediaKind;
}

async function findCachedVoiceBackgroundMedia(
	id: string,
): Promise<{path: string; mimeType: string; mediaKind: VoiceBackgroundMediaKind} | null> {
	const cacheDir = getCacheDir();
	const cacheKey = getCacheKey(id);
	const entries = await fs.readdir(cacheDir, {withFileTypes: true}).catch(() => []);
	for (const entry of entries) {
		if (!entry.isFile()) continue;
		const extension = path.extname(entry.name).slice(1);
		if (!entry.name.startsWith(`${cacheKey}.`)) continue;
		const mediaType = MEDIA_TYPES_BY_EXTENSION.get(extension);
		if (!mediaType) continue;
		const filePath = path.join(cacheDir, entry.name);
		let mediaKind = mediaType.mediaKind;
		if (mediaType.extension === 'webp') {
			const data = await fs.readFile(filePath).catch(() => null);
			if (!data) continue;
			mediaKind = classifyMediaKind(mediaType, data);
		}
		return {
			path: filePath,
			mimeType: mediaType.mimeType,
			mediaKind,
		};
	}
	return null;
}

async function trimVoiceBackgroundMediaCache(cacheDir: string): Promise<void> {
	const entries = await fs.readdir(cacheDir, {withFileTypes: true});
	const files = await Promise.all(
		entries
			.filter((entry) => entry.isFile())
			.map(async (entry) => {
				const filePath = path.join(cacheDir, entry.name);
				const stats = await fs.stat(filePath);
				return {filePath, mtimeMs: stats.mtimeMs};
			}),
	);
	if (files.length <= MAX_BACKGROUND_MEDIA_CACHE_FILES) return;
	files.sort((a, b) => a.mtimeMs - b.mtimeMs);
	const deleteCount = files.length - MAX_BACKGROUND_MEDIA_CACHE_FILES;
	await Promise.all(files.slice(0, deleteCount).map(({filePath}) => fs.rm(filePath, {force: true})));
}

async function cacheVoiceBackgroundMedia(
	options: VoiceBackgroundMediaCacheRequest,
): Promise<VoiceBackgroundMediaCacheResult> {
	const cacheDir = getCacheDir();
	const cacheKey = getCacheKey(options.id);
	const mediaType = getMediaType(options);
	const data = Buffer.from(options.data);
	if (data.byteLength === 0 || data.byteLength > MAX_BACKGROUND_MEDIA_BYTES) {
		throw new Error('Voice background media size is invalid');
	}
	const mediaKind = classifyMediaKind(mediaType, data);
	await fs.mkdir(cacheDir, {recursive: true});
	const filePath = path.join(cacheDir, `${cacheKey}.${mediaType.extension}`);
	const tempPath = path.join(cacheDir, `${cacheKey}.${process.pid}.${Date.now()}.tmp`);
	await fs.writeFile(tempPath, data, {mode: 0o600});
	try {
		await fs.rename(tempPath, filePath);
	} catch (error) {
		await fs.rm(tempPath, {force: true});
		throw error;
	}
	await trimVoiceBackgroundMediaCache(cacheDir);
	return {path: filePath, mediaKind};
}

async function resolveVoiceBackgroundMedia(id: string): Promise<VoiceBackgroundMediaCacheResult | null> {
	const cached = await findCachedVoiceBackgroundMedia(id);
	if (!cached) return null;
	return {path: cached.path, mediaKind: cached.mediaKind};
}

async function readVoiceBackgroundMedia(id: string): Promise<VoiceBackgroundMediaReadResult | null> {
	const cached = await findCachedVoiceBackgroundMedia(id);
	if (!cached) return null;
	const data = await fs.readFile(cached.path);
	if (data.byteLength === 0 || data.byteLength > MAX_BACKGROUND_MEDIA_BYTES) {
		throw new Error('Voice background media size is invalid');
	}
	return {
		path: cached.path,
		mediaKind: cached.mediaKind,
		dataUrl: `data:${cached.mimeType};base64,${data.toString('base64')}`,
	};
}

async function deleteVoiceBackgroundMedia(id: string): Promise<void> {
	const cacheDir = getCacheDir();
	const cacheKey = getCacheKey(id);
	const entries = await fs.readdir(cacheDir, {withFileTypes: true}).catch(() => []);
	const targets = entries
		.filter((entry) => entry.isFile())
		.filter((entry) => entry.name.startsWith(`${cacheKey}.`))
		.map((entry) => fs.rm(path.join(cacheDir, entry.name), {force: true}));
	await Promise.all(targets);
}

export function registerVoiceBackgroundMediaCacheHandlers(): void {
	ipcMain.handle('voice-background-media-cache:write', (_event, options: VoiceBackgroundMediaCacheRequest) =>
		cacheVoiceBackgroundMedia(options),
	);
	ipcMain.handle('voice-background-media-cache:resolve', (_event, id: string) => resolveVoiceBackgroundMedia(id));
	ipcMain.handle('voice-background-media-cache:read', (_event, id: string) => readVoiceBackgroundMedia(id));
	ipcMain.handle('voice-background-media-cache:delete', (_event, id: string) => deleteVoiceBackgroundMedia(id));
}
