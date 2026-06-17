// SPDX-License-Identifier: AGPL-3.0-or-later

import {startServiceInitialization} from '@pkgs/initialization/src/Init';
import type {ShutdownFn} from '@pkgs/initialization/src/ServiceInitializationTypes';

interface CreateServiceInstrumentationOptions {
	serviceName: string;
	config: {
		env: string;
	};
}

export function createServiceInstrumentation(options: CreateServiceInstrumentationOptions): ShutdownFn {
	const {serviceName, config} = options;
	return startServiceInitialization({
		serviceName,
		environment: config.env,
	});
}
