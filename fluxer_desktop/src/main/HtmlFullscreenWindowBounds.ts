// SPDX-License-Identifier: AGPL-3.0-or-later

export interface HtmlFullscreenWindowBounds {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface HtmlFullscreenWindowBoundsRestoreState {
	previousBounds: HtmlFullscreenWindowBounds;
	currentBounds: HtmlFullscreenWindowBounds;
	displayBounds: HtmlFullscreenWindowBounds;
	wasMaximized: boolean;
	isMaximized: boolean;
}

const DISPLAY_BOUNDS_TOLERANCE_PX = 2;

function boundsCoverDisplay(bounds: HtmlFullscreenWindowBounds, displayBounds: HtmlFullscreenWindowBounds): boolean {
	return (
		bounds.x <= displayBounds.x + DISPLAY_BOUNDS_TOLERANCE_PX &&
		bounds.y <= displayBounds.y + DISPLAY_BOUNDS_TOLERANCE_PX &&
		bounds.x + bounds.width >= displayBounds.x + displayBounds.width - DISPLAY_BOUNDS_TOLERANCE_PX &&
		bounds.y + bounds.height >= displayBounds.y + displayBounds.height - DISPLAY_BOUNDS_TOLERANCE_PX
	);
}

export function shouldRestoreHtmlFullscreenWindowBounds(state: HtmlFullscreenWindowBoundsRestoreState): boolean {
	if (state.wasMaximized || state.isMaximized) return false;
	if (boundsCoverDisplay(state.previousBounds, state.displayBounds)) return false;
	return boundsCoverDisplay(state.currentBounds, state.displayBounds);
}
