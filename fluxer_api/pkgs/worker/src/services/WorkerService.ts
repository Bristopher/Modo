// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {IWorkerService} from '@pkgs/worker/src/contracts/IWorkerService';
import type {TracingInterface, WorkerJobOptions, WorkerJobPayload} from '@pkgs/worker/src/contracts/WorkerTypes';
import type {IQueueProvider} from '@pkgs/worker/src/providers/IQueueProvider';
import {createQueueProvider} from '@pkgs/worker/src/providers/QueueProviderFactory';

export interface WorkerServiceOptions {
	queueBaseUrl?: string | undefined;
	queueProvider?: IQueueProvider | undefined;
	logger: LoggerInterface;
	tracing?: TracingInterface | undefined;
	timeoutMs?: number | undefined;
}

export class WorkerService implements IWorkerService {
	private readonly queue: IQueueProvider;
	private readonly logger: LoggerInterface;

	constructor(options: WorkerServiceOptions) {
		this.queue = createQueueProvider({
			queueProvider: options.queueProvider,
			queueBaseUrl: options.queueBaseUrl,
			timeoutMs: options.timeoutMs,
			tracing: options.tracing,
		});
		this.logger = options.logger;
	}

	async addJob<TPayload extends WorkerJobPayload = WorkerJobPayload>(
		taskType: string,
		payload: TPayload,
		options?: WorkerJobOptions,
	): Promise<bigint> {
		try {
			await this.queue.enqueue(taskType, payload, {
				runAt: options?.runAt,
				maxAttempts: options?.maxAttempts,
				priority: options?.priority,
			});
			this.logger.debug({taskType, payload}, 'Job queued successfully');
		} catch (error) {
			this.logger.error({error, taskType, payload}, 'Failed to queue job');
			throw error;
		}
		return 0n;
	}

	async cancelJob(jobId: bigint): Promise<boolean> {
		try {
			const cancelled = await this.queue.cancelJob(jobId.toString());
			if (cancelled) {
				this.logger.info({jobId: jobId.toString()}, 'Job cancelled successfully');
			} else {
				this.logger.debug({jobId: jobId.toString()}, 'Job not found (may have already been processed)');
			}
			return cancelled;
		} catch (error) {
			this.logger.error({error, jobId: jobId.toString()}, 'Failed to cancel job');
			throw error;
		}
	}

	async retryDeadLetterJob(jobId: bigint): Promise<boolean> {
		try {
			const retried = await this.queue.retryDeadLetterJob(jobId.toString());
			if (retried) {
				this.logger.info({jobId: jobId.toString()}, 'Dead letter job retried successfully');
			} else {
				this.logger.debug({jobId: jobId.toString()}, 'Job not found in dead letter queue');
			}
			return retried;
		} catch (error) {
			this.logger.error({error, jobId: jobId.toString()}, 'Failed to retry dead letter job');
			throw error;
		}
	}

	getQueue(): IQueueProvider {
		return this.queue;
	}
}
