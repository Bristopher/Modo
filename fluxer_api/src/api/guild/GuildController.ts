// SPDX-License-Identifier: AGPL-3.0-or-later

import type {HonoApp} from '../types/HonoEnv';
import {registerGuildControllers} from './controllers/index';

export function GuildController(app: HonoApp) {
	registerGuildControllers(app);
}
