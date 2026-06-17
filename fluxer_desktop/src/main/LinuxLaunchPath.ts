// SPDX-License-Identifier: AGPL-3.0-or-later

export function getStableLinuxLaunchPath(): string {
	if (process.platform === 'linux') {
		const appImage = process.env.APPIMAGE;
		if (appImage && appImage.length > 0) return appImage;
	}
	return process.execPath;
}

export function getStableRelaunchOptions(): {execPath: string} | undefined {
	if (process.platform !== 'linux') return undefined;
	const execPath = getStableLinuxLaunchPath();
	if (execPath === process.execPath) return undefined;
	return {execPath};
}
