// SPDX-License-Identifier: AGPL-3.0-or-later

import {createChildLogger} from '@electron/common/Logger';
import {onLocaleChange, t} from '@electron/main/MainI18n';
import {getMainWindow, showWindow} from '@electron/main/Window';
import {app, Menu} from 'electron';

const logger = createChildLogger('DockMenu');

function dispatchToRenderer(channel: string): void {
	const mainWindow = getMainWindow();
	if (mainWindow && !mainWindow.isDestroyed()) {
		showWindow();
		mainWindow.webContents.send(channel);
	}
}

function buildDockMenu(): Menu {
	return Menu.buildFromTemplate([
		{
			label: t('desktop.jumpList.openSettings'),
			click: () => dispatchToRenderer('open-settings'),
		},
		{
			label: t('desktop.jumpList.newDirectMessage'),
			click: () => dispatchToRenderer('jump-list-new-dm'),
		},
	]);
}

function refreshDockMenu(): void {
	if (process.platform !== 'darwin') return;
	if (!app.dock) return;
	try {
		app.dock.setMenu(buildDockMenu());
	} catch (error) {
		logger.warn('Failed to install macOS dock menu', {error});
	}
}

let installed = false;

export function initializeDockMenu(): void {
	if (process.platform !== 'darwin') return;
	if (installed) return;
	installed = true;
	refreshDockMenu();
	onLocaleChange(refreshDockMenu);
}
