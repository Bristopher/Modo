// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomBytes} from 'node:crypto';

const OAUTH_TOKEN_SECRET_BYTES = 32;

export function generateOAuthTokenSecret(): string {
	return randomBytes(OAUTH_TOKEN_SECRET_BYTES).toString('base64url');
}
