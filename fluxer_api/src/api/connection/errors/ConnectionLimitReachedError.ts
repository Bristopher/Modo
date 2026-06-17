// SPDX-License-Identifier: AGPL-3.0-or-later

import {APIErrorCodes} from '@fluxer/constants/src/ApiErrorCodes';
import {MAX_CONNECTIONS_PER_USER} from '@fluxer/constants/src/ConnectionConstants';
import {ThrottledError} from '@fluxer/errors/src/domains/core/ThrottledError';

export class ConnectionLimitReachedError extends ThrottledError {
	constructor(limit: number = MAX_CONNECTIONS_PER_USER) {
		super({
			code: APIErrorCodes.CONNECTION_LIMIT_REACHED,
			messageVariables: {limit},
		});
	}
}
