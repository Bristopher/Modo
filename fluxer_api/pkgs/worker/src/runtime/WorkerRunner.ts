// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomUUID} from 'node:crypto';
import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import type {LeasedQueueJob, TracingInterface} from '@pkgs/worker/src/contracts/WorkerTypes';
import type {IQueueProvider} from '@pkgs/worker/src/providers/IQueueProvider';
import {createQueueProvider} from '@pkgs/worker/src/providers/QueueProviderFactory';
import {WorkerService} from '@pkgs/worker/src/services/WorkerService';
import {ms} from 'itty-time';

export interface WorkerRunnerOptions {
	tasks: Record<string, WorkerTaskHandler>;
	queueBaseUrl?: string | undefined;
	queueProvider?: IQueueProvider | undefined;
	logger: LoggerInterface;
	workerId?: string | undefined;
	taskTypes?: Array<string> | undefined;
	concurrency?: number | undefined;
	tracing?: TracingInterface | undefined;
	requestTimeoutMs?: number | undefined;
}

export class WorkerRunner {
	private readonly tasks: Record<string, WorkerTaskHandler>;
	private readonly workerId: string;
	private readonly taskTypes: Array<string>;
	private readonly concurrency: number;
	private readonly queue: IQueueProvider;
	private readonly workerService: WorkerService;
	private readonly logger: LoggerInterface;
	private readonly tracing: TracingInterface | undefined;
	private running = false;
	private abortController: AbortController | null = null;
	private workerLoopPromises: Array<Promise<void>> = [];

	constructor(options: WorkerRunnerOptions) {
		this.tasks = options.tasks;
		this.workerId = options.workerId ?? `worker-${randomUUID()}`;
		this.taskTypes = options.taskTypes ?? Object.keys(options.tasks);
		this.concurrency = options.concurrency ?? 1;
		this.logger = options.logger;
		this.tracing = options.tracing;
		this.queue = createQueueProvider({
			queueProvider: options.queueProvider,
			queueBaseUrl: options.queueBaseUrl,
			timeoutMs: options.requestTimeoutMs,
			tracing: options.tracing,
		});
		this.workerService = new WorkerService({
			queueProvider: this.queue,
			logger: options.logger,
		});
	}

	async start(): Promise<void> {
		if (this.running) {
			this.logger.warn({workerId: this.workerId}, 'Worker already running');
			return;
		}
		this.running = true;
		this.abortController = new AbortController();
		this.logger.info(
			{workerId: this.workerId, taskTypes: this.taskTypes, concurrency: this.concurrency},
			'Worker starting',
		);
		this.workerLoopPromises = Array.from({length: this.concurrency}, (_, i) =>
			this.workerLoop(i, this.abortController!.signal),
		);
		Promise.all(this.workerLoopPromises).catch((error) => {
			this.logger.error({workerId: this.workerId, error}, 'Worker loop failed unexpectedly');
		});
	}

	async stop(): Promise<void> {
		if (!this.running) {
			return;
		}
		this.running = false;
		this.abortController?.abort();
		const stopTimeout = new Promise<void>((resolve) => setTimeout(resolve, ms('2 seconds')));
		await Promise.race([Promise.all(this.workerLoopPromises), stopTimeout]);
		this.workerLoopPromises = [];
		this.logger.info({workerId: this.workerId}, 'Worker stopped');
	}

	async processJob(leasedJob: LeasedQueueJob): Promise<void> {
		await this.executeJob(leasedJob);
	}

	getWorkerService(): WorkerService {
		return this.workerService;
	}

	getQueue(): IQueueProvider {
		return this.queue;
	}

	isRunning(): boolean {
		return this.running;
	}

	private async workerLoop(workerIndex: number, signal: AbortSignal): Promise<void> {
		this.logger.info({workerId: this.workerId, workerIndex}, 'Worker loop started');
		while (!signal.aborted) {
			try {
				const leasedJobs = await this.queue.dequeue(this.taskTypes, 1);
				if (!leasedJobs || leasedJobs.length === 0) {
					await this.sleep(100);
					continue;
				}
				const leasedJob = leasedJobs[0]!;
				const job = leasedJob.job;
				this.logger.info(
					{
						workerId: this.workerId,
						workerIndex,
						jobId: job.id,
						taskType: job.task_type,
						attempts: job.attempts,
						receipt: leasedJob.receipt,
					},
					'Processing job',
				);
				await this.executeJob(leasedJob);
				this.logger.info({workerId: this.workerId, workerIndex, jobId: job.id}, 'Job completed successfully');
			} catch (error) {
				this.logger.error({workerId: this.workerId, workerIndex, error}, 'Worker loop error');
				await this.sleep(ms('1 second'));
			}
		}
		this.logger.info({workerId: this.workerId, workerIndex}, 'Worker loop stopped');
	}

	private async executeJob(leasedJob: LeasedQueueJob): Promise<void> {
		const execute = async () => {
			const task = this.tasks[leasedJob.job.task_type];
			if (!task) {
				throw new Error(`Unknown task: ${leasedJob.job.task_type}`);
			}
			this.tracing?.addSpanEvent('job.execution.start');
			try {
				await task(leasedJob.job.payload, {
					logger: this.logger.child({jobId: leasedJob.job.id}),
					jobId: 0n,
					addJob: this.workerService.addJob.bind(this.workerService),
					reportProgress: async () => {},
					shouldCancel: async () => false,
					setContextLink: async () => {},
				});
				this.tracing?.addSpanEvent('job.execution.success');
				this.tracing?.setSpanAttributes({'job.status': 'success'});
				await this.queue.complete(leasedJob.receipt);
			} catch (error) {
				this.logger.error({jobId: leasedJob.job.id, error}, 'Job failed');
				this.tracing?.setSpanAttributes({
					'job.status': 'failed',
					'job.error': error instanceof Error ? error.message : String(error),
				});
				this.tracing?.addSpanEvent('job.execution.failed', {
					error: error instanceof Error ? error.message : String(error),
				});
				await this.queue.fail(leasedJob.receipt, String(error));
			}
		};
		if (this.tracing) {
			await this.tracing.withSpan(
				{
					name: 'worker.process_job',
					attributes: {
						'worker.id': this.workerId,
						'job.id': leasedJob.job.id,
						'job.task_type': leasedJob.job.task_type,
						'job.attempts': leasedJob.job.attempts,
					},
				},
				execute,
			);
		} else {
			await execute();
		}
	}

	private async sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}
}
