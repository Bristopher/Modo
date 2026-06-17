// SPDX-License-Identifier: AGPL-3.0-or-later

import {BUILD_CHANNEL} from '@electron/common/BuildChannel';
import log from 'electron-log';

log.transports.file.level = BUILD_CHANNEL === 'canary' ? 'debug' : 'info';

log.transports.console.level = BUILD_CHANNEL === 'canary' ? 'debug' : 'info';

export const Logger = {
	debug: (...args: Array<unknown>) => log.debug(...args),
	info: (...args: Array<unknown>) => log.info(...args),
	warn: (...args: Array<unknown>) => log.warn(...args),
	error: (...args: Array<unknown>) => log.error(...args),
};

export function createChildLogger(componentName: string): typeof Logger {
	const prefix = `[${componentName}]`;
	return {
		debug: (...args: Array<unknown>) => log.debug(prefix, ...args),
		info: (...args: Array<unknown>) => log.info(prefix, ...args),
		warn: (...args: Array<unknown>) => log.warn(prefix, ...args),
		error: (...args: Array<unknown>) => log.error(prefix, ...args),
	};
}
