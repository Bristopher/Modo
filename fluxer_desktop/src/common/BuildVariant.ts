// SPDX-License-Identifier: AGPL-3.0-or-later

import type {DesktopBuildVariant} from './Types';

const EMBEDDED_DESKTOP_BUILD_VARIANT = process.env.FLUXER_DESKTOP_BUILD_VARIANT;

export const DESKTOP_BUILD_VARIANT: DesktopBuildVariant =
	EMBEDDED_DESKTOP_BUILD_VARIANT === 'windows-game-capture' ? 'windows-game-capture' : 'default';

export const IS_WINDOWS_GAME_CAPTURE_BUILD = DESKTOP_BUILD_VARIANT === 'windows-game-capture';
