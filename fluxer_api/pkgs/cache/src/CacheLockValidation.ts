// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomBytes} from 'node:crypto';

const LOCK_KEY_PATTERN = /^[a-zA-Z0-9:_-]+$/;
const LOCK_TOKEN_PATTERN = /^[a-z0-9]+$/;

export function validateLockKey(key: string): void {
	if (!LOCK_KEY_PATTERN.test(key)) {
		throw new Error('Invalid lock key format');
	}
}

export function validateLockToken(token: string): void {
	if (!LOCK_TOKEN_PATTERN.test(token)) {
		throw new Error('Invalid lock token format');
	}
}

export function generateLockToken(): string {
	return randomBytes(16).toString('hex');
}

export function formatLockKey(key: string): string {
	return `lock:${key}`;
}
