// SPDX-License-Identifier: AGPL-3.0-or-later

import {IS_WINDOWS_GAME_CAPTURE_BUILD} from '@electron/common/BuildVariant';

const WINDOWS_GAME_CAPTURE_MODULE_ENV = 'FLUXER_WINDOWS_GAME_CAPTURE_MODULE_ENABLED';
export const WINDOWS_GAME_CAPTURE_DISABLED_DETAIL = 'windows-game-capture-disabled-until-code-signed';
export const WINDOWS_GAME_CAPTURE_DISABLED_REASON = 'disabled-by-launch';

export const WINDOWS_GAME_CAPTURE_MODULE_ENABLED =
	IS_WINDOWS_GAME_CAPTURE_BUILD || process.env.FLUXER_WINDOWS_GAME_CAPTURE_MODULE_ENABLED === 'true';

export function enableWindowsGameCaptureModuleForCurrentProcess(): void {
	if (!WINDOWS_GAME_CAPTURE_MODULE_ENABLED) return;
	process.env[WINDOWS_GAME_CAPTURE_MODULE_ENV] = 'true';
}
