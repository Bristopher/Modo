// SPDX-License-Identifier: AGPL-3.0-or-later

import {assertNonEmptyString} from '@pkgs/rate_limit/src/internal/RateLimitValidation';

const RATE_LIMIT_PREFIX = 'ratelimit';
const BUCKET_NAMESPACE = 'bucket';
const GLOBAL_NAMESPACE = 'global';

interface IRateLimitKeyFactory {
	getIdentifierKey(identifier: string): string;
	getBucketKey(bucket: string): string;
	getGlobalKey(identifier: string): string;
}

export class RateLimitKeyFactory implements IRateLimitKeyFactory {
	getIdentifierKey(identifier: string): string {
		assertNonEmptyString(identifier, 'identifier');
		return `${RATE_LIMIT_PREFIX}:${identifier}`;
	}

	getBucketKey(bucket: string): string {
		assertNonEmptyString(bucket, 'bucket');
		return `${RATE_LIMIT_PREFIX}:${BUCKET_NAMESPACE}:${bucket}`;
	}

	getGlobalKey(identifier: string): string {
		assertNonEmptyString(identifier, 'identifier');
		return `${RATE_LIMIT_PREFIX}:${GLOBAL_NAMESPACE}:${identifier}`;
	}
}
