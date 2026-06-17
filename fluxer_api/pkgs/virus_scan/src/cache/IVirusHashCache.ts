// SPDX-License-Identifier: AGPL-3.0-or-later

export interface IVirusHashCache {
	isKnownVirusHash(fileHash: string): Promise<boolean>;
	cacheVirusHash(fileHash: string): Promise<void>;
}
