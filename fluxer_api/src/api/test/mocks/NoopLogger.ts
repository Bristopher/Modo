// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {ILogger} from '../../ILogger';

function noop(): void {}

export class NoopLogger implements ILogger, LoggerInterface {
	trace = noop;
	debug = noop;
	info = noop;
	warn = noop;
	error = noop;
	fatal = noop;

	child(): this {
		return this;
	}
}
