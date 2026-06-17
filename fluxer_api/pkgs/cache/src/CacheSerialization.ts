// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CacheLogger} from '@pkgs/cache/src/CacheProviderTypes';

export function safeJsonParse<T>(value: string, logger?: CacheLogger): T | null {
	try {
		return JSON.parse(value);
	} catch (error) {
		if (logger) {
			const truncatedValue = value.length > 200 ? `${value.substring(0, 200)}...` : value;
			const errorMessage = error instanceof Error ? error.message : String(error);
			logger.error({errorMessage, value: truncatedValue}, '[CacheProvider] JSON parse error');
		}
		return null;
	}
}

export function serializeValue<T>(value: T): string {
	return JSON.stringify(value);
}
