// SPDX-License-Identifier: AGPL-3.0-or-later

export interface BulkProgressHelpers {
	reportProgress: (current: number, total: number, message?: string | null) => Promise<void>;
	shouldCancel: () => Promise<boolean>;
}

export class BulkCancelledError extends Error {
	constructor() {
		super('Bulk operation cancelled');
		this.name = 'BulkCancelledError';
	}
}
