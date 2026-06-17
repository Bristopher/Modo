// SPDX-License-Identifier: AGPL-3.0-or-later

import {createRequire} from 'node:module';
import {repairWindowsShortcuts} from '@electron/main/WindowsShortcuts';

const requireModule = createRequire(import.meta.url);

if (process.platform === 'win32') {
	const {VelopackApp} = requireModule('velopack') as typeof import('velopack');
	VelopackApp.build()
		.onAfterInstallFastCallback(() => {
			repairWindowsShortcuts();
		})
		.onAfterUpdateFastCallback(() => {
			repairWindowsShortcuts();
		})
		.onFirstRun(() => {
			repairWindowsShortcuts();
		})
		.onRestarted(() => {
			repairWindowsShortcuts();
		})
		.run();
	repairWindowsShortcuts();
}

await import(new URL('./MainApp.js', import.meta.url).href);
