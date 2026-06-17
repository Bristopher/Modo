// SPDX-License-Identifier: AGPL-3.0-or-later

import type {EnqueueOptions, LeasedQueueJob, WorkerJobPayload} from '@pkgs/worker/src/contracts/WorkerTypes';

export interface IQueueProvider {
	enqueue(taskType: string, payload: WorkerJobPayload, options?: EnqueueOptions): Promise<string>;
	dequeue(taskTypes: Array<string>, limit?: number): Promise<Array<LeasedQueueJob>>;
	upsertCron(id: string, taskType: string, payload: WorkerJobPayload, cronExpression: string): Promise<void>;
	complete(receipt: string): Promise<void>;
	fail(receipt: string, error: string): Promise<void>;
	cancelJob(jobId: string): Promise<boolean>;
	retryDeadLetterJob(jobId: string): Promise<boolean>;
}
