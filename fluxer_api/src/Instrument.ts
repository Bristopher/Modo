// SPDX-License-Identifier: AGPL-3.0-or-later

import {Config} from '@app/Config';
import {createServiceInstrumentation} from '@pkgs/initialization/src/CreateServiceInstrumentation';

export const shutdownInstrumentation = createServiceInstrumentation({
	serviceName: 'fluxer-api',
	config: Config,
});
