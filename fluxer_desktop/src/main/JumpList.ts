// SPDX-License-Identifier: AGPL-3.0-or-later

import {createChildLogger} from '@electron/common/Logger';
import {onLocaleChange, t} from '@electron/main/MainI18n';
import {app} from 'electron';

const logger = createChildLogger('JumpList');
export const TASK_ARG_PREFIX = '--fluxer-task=';

type JumpListTaskId = 'open-settings' | 'new-dm';

function buildTasks(): Array<Electron.Task> {
	const exePath = process.execPath;
	return [
		{
			program: exePath,
			arguments: `${TASK_ARG_PREFIX}open-settings`,
			iconPath: exePath,
			iconIndex: 0,
			title: t('desktop.jumpList.openSettings'),
			description: t('desktop.jumpList.openSettingsDescription'),
		},
		{
			program: exePath,
			arguments: `${TASK_ARG_PREFIX}new-dm`,
			iconPath: exePath,
			iconIndex: 0,
			title: t('desktop.jumpList.newDirectMessage'),
			description: t('desktop.jumpList.newDirectMessageDescription'),
		},
	];
}

let installed = false;

function refreshJumpList(): void {
	if (process.platform !== 'win32') return;
	try {
		const ok = app.setUserTasks(buildTasks());
		if (!ok) {
			logger.warn('setUserTasks returned false — JumpList may be unavailable on this Windows version');
		}
	} catch (error) {
		logger.warn('Failed to install Windows JumpList', {error});
	}
}

export function initializeJumpList(): void {
	if (process.platform !== 'win32') return;
	if (installed) return;
	installed = true;
	refreshJumpList();
	onLocaleChange(refreshJumpList);
}

export function parseJumpListTaskFromArgv(argv: ReadonlyArray<string>): JumpListTaskId | null {
	for (const arg of argv) {
		if (!arg.startsWith(TASK_ARG_PREFIX)) continue;
		const id = arg.slice(TASK_ARG_PREFIX.length);
		if (id === 'open-settings' || id === 'new-dm') return id;
	}
	return null;
}
