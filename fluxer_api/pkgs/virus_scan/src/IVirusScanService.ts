// SPDX-License-Identifier: AGPL-3.0-or-later

import type {VirusScanResult} from '@pkgs/virus_scan/src/VirusScanResult';

export interface IVirusScanService {
	readonly enabled: boolean;
	initialize(): Promise<void>;
	scanFile(filePath: string): Promise<VirusScanResult>;
	scanBuffer(buffer: Buffer, filename: string): Promise<VirusScanResult>;
}
