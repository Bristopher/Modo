// SPDX-License-Identifier: AGPL-3.0-or-later

import {getMainWindow} from '@electron/main/Window';

export type TaskbarProgressMode = 'normal' | 'indeterminate' | 'error' | 'paused' | 'none';

export function setTaskbarProgress(fraction: number, mode: TaskbarProgressMode = 'normal'): void {
	const mainWindow = getMainWindow();
	if (!mainWindow || mainWindow.isDestroyed()) return;
	try {
		if (mode === 'none' || fraction < 0) {
			mainWindow.setProgressBar(-1);
			return;
		}
		const clamped = Math.max(0, Math.min(1, fraction));
		mainWindow.setProgressBar(clamped, {mode});
	} catch {}
}
