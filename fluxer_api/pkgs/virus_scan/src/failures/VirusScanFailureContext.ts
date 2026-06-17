// SPDX-License-Identifier: AGPL-3.0-or-later

export interface VirusScanFailureContext {
	error: unknown;
	filename: string;
	fileHash: string;
	failOpen: boolean;
}
