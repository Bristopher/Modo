// SPDX-License-Identifier: AGPL-3.0-or-later

import fs from 'node:fs';

const FLATPAK_INFO_PATH = '/.flatpak-info';

export function isFlatpakRuntime(): boolean {
	if (process.platform !== 'linux') return false;
	if (process.env.container === 'flatpak') return true;
	if (process.env.FLATPAK_ID && process.env.FLATPAK_ID.length > 0) return true;
	try {
		return fs.existsSync(FLATPAK_INFO_PATH);
	} catch {
		return false;
	}
}

export function getFlatpakAppId(): string | null {
	if (!isFlatpakRuntime()) return null;
	const id = process.env.FLATPAK_ID?.trim();
	return id && id.length > 0 ? id : null;
}
