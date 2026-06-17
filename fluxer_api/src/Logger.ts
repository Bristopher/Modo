// SPDX-License-Identifier: AGPL-3.0-or-later

import {createLogger, type Logger as FluxerLogger} from '@fluxer/logger/src/Logger';

export const Logger = createLogger('fluxer-api');

export type Logger = FluxerLogger;
