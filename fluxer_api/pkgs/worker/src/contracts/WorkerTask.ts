// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {WorkerJobOptions, WorkerJobPayload} from '@pkgs/worker/src/contracts/WorkerTypes';

export interface WorkerTaskHelpers {
	logger: LoggerInterface;
	jobId: bigint;
	addJob: <TPayload extends WorkerJobPayload = WorkerJobPayload>(
		taskType: string,
		payload: TPayload,
		options?: WorkerJobOptions,
	) => Promise<bigint>;
	reportProgress: (current: number, total: number | null, message?: string | null) => Promise<void>;
	shouldCancel: () => Promise<boolean>;
	setContextLink: (link: string) => Promise<void>;
}

export type WorkerTaskHandler<Payload = Record<string, unknown>> = (
	payload: Payload,
	helpers: WorkerTaskHelpers,
) => Promise<void>;

export class JobCancelledError extends Error {
	constructor(message = 'Job cancelled by admin') {
		super(message);
		this.name = 'JobCancelledError';
	}
}
