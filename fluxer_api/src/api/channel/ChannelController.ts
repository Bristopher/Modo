// SPDX-License-Identifier: AGPL-3.0-or-later

import type {HonoApp} from '../types/HonoEnv';
import {registerChannelControllers} from './controllers/index';

export function ChannelController(app: HonoApp) {
	registerChannelControllers(app);
}
