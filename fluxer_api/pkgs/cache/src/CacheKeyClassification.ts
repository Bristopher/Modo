// SPDX-License-Identifier: AGPL-3.0-or-later

export function classifyKeyType(key: string): string {
	if (key.startsWith('lock:')) return 'lock';
	if (key.startsWith('bluesky:')) return 'bluesky';
	if (key.includes(':session:')) return 'session';
	if (key.includes(':user:')) return 'user';
	if (key.includes(':guild:')) return 'guild';
	if (key.includes(':channel:')) return 'channel';
	if (key.includes(':ratelimit:')) return 'ratelimit';
	return 'other';
}
