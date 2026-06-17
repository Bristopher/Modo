// SPDX-License-Identifier: AGPL-3.0-or-later

import type {HonoApp} from '../../types/HonoEnv';
import {PackController} from './PackController';
import {PackEmojiController} from './PackEmojiController';
import {PackStickerController} from './PackStickerController';

export function registerPackControllers(app: HonoApp) {
	PackController(app);
	PackEmojiController(app);
	PackStickerController(app);
}
