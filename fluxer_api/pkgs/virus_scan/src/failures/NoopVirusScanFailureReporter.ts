// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IVirusScanFailureReporter} from '@pkgs/virus_scan/src/failures/IVirusScanFailureReporter';
import type {VirusScanFailureContext} from '@pkgs/virus_scan/src/failures/VirusScanFailureContext';

export class NoopVirusScanFailureReporter implements IVirusScanFailureReporter {
	async initialize(): Promise<void> {}

	async reportFailure(_context: VirusScanFailureContext): Promise<void> {}
}
