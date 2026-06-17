// SPDX-License-Identifier: AGPL-3.0-or-later

const VERSION_PATTERN = /^\/v\d+/;

export function stripApiPrefix(path: string): string {
	if (path === '/api') {
		return '/';
	}
	if (path.startsWith('/api/')) {
		const afterApi = path.slice(4);
		if (VERSION_PATTERN.test(afterApi)) {
			const versionMatch = afterApi.match(VERSION_PATTERN);
			if (versionMatch) {
				const remaining = afterApi.slice(versionMatch[0].length);
				return remaining === '' ? '/' : remaining;
			}
		}
		return afterApi;
	}
	if (VERSION_PATTERN.test(path)) {
		const versionMatch = path.match(VERSION_PATTERN);
		if (versionMatch) {
			const remaining = path.slice(versionMatch[0].length);
			return remaining === '' ? '/' : remaining;
		}
	}
	return path;
}

export function normalizeRequestPath(path: string): string {
	let normalized = stripApiPrefix(path);
	if (normalized.length > 1 && normalized.endsWith('/')) {
		normalized = normalized.slice(0, -1);
	}
	return normalized;
}
