// SPDX-License-Identifier: AGPL-3.0-or-later

import type {VirusScanFailureContext} from '@pkgs/virus_scan/src/failures/VirusScanFailureContext';

export interface IVirusScanFailureReporter {
	initialize(): Promise<void>;
	reportFailure(context: VirusScanFailureContext): Promise<void>;
}
