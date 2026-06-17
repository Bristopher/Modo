// SPDX-License-Identifier: AGPL-3.0-or-later

import type {VirusScanProviderResult} from '@pkgs/virus_scan/src/VirusScanProviderResult';

export interface IVirusScanProvider {
	scanFile(filePath: string): Promise<VirusScanProviderResult>;
	scanBuffer(buffer: Buffer): Promise<VirusScanProviderResult>;
}
