// SPDX-License-Identifier: AGPL-3.0-or-later

import {buildAPIConfigFromMaster} from '@app/api/Config';
import type {APIConfig} from '@app/api/config/APIConfig';
import {loadConfig} from '@fluxer/config/src/ConfigLoader';

const master = await loadConfig();
const apiConfig = buildAPIConfigFromMaster(master);

interface ExtendedAPIConfig extends APIConfig {
	env: string;
}

export const Config: ExtendedAPIConfig = {
	env: master.env,
	...apiConfig,
};

export type Config = typeof Config;
