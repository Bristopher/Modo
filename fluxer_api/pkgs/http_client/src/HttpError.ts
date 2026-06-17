// SPDX-License-Identifier: AGPL-3.0-or-later

export type HttpErrorType = 'aborted' | 'network_error' | 'unknown';

export class HttpError extends Error {
	constructor(
		message: string,
		public readonly status?: number,
		public readonly response?: Response,
		public readonly isExpected = false,
		public readonly errorType?: HttpErrorType,
	) {
		super(message);
		this.name = 'HttpError';
	}
}
