// SPDX-License-Identifier: AGPL-3.0-or-later

import {getMainWindow} from '@electron/main/Window';

export function flashWindowForAttention(persistent: boolean): void {
	if (process.platform === 'darwin') return;
	const mainWindow = getMainWindow();
	if (!mainWindow || mainWindow.isDestroyed()) return;
	if (mainWindow.isFocused()) return;
	try {
		mainWindow.flashFrame(persistent);
	} catch {}
}

export function stopFlashingWindow(): void {
	const mainWindow = getMainWindow();
	if (!mainWindow || mainWindow.isDestroyed()) return;
	try {
		mainWindow.flashFrame(false);
	} catch {}
}
