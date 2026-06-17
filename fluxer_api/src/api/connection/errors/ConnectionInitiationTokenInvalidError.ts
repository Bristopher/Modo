// SPDX-License-Identifier: AGPL-3.0-or-later

import {APIErrorCodes} from '@fluxer/constants/src/ApiErrorCodes';
import {BadRequestError} from '@fluxer/errors/src/domains/core/BadRequestError';

export class ConnectionInitiationTokenInvalidError extends BadRequestError {
	constructor() {
		super({code: APIErrorCodes.CONNECTION_INITIATION_TOKEN_INVALID});
	}
}
