// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import {setWorkerDependencies} from '@pkgs/worker/src/context/WorkerContext';
import type {IWorkerService} from '@pkgs/worker/src/contracts/IWorkerService';
import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import type {
	LeasedQueueJob,
	TracingInterface,
	WorkerConfig,
	WorkerQueueConfig,
	WorkerRuntimeConfig,
} from '@pkgs/worker/src/contracts/WorkerTypes';
import type {IQueueProvider} from '@pkgs/worker/src/providers/IQueueProvider';
import {WorkerRunner} from '@pkgs/worker/src/runtime/WorkerRunner';
import {WorkerTaskRegistry} from '@pkgs/worker/src/runtime/WorkerTaskRegistry';
import {WorkerService} from '@pkgs/worker/src/services/WorkerService';

export interface CreateWorkerOptions {
	queue: WorkerQueueOptions;
	runtime?: WorkerRuntimeConfig | undefined;
	logger: LoggerInterface;
	dependencies?: unknown;
	taskRegistry?: WorkerTaskRegistry | undefined;
	tracing?: TracingInterface | undefined;
}

export interface CreateWorkerLegacyOptions {
	config: WorkerConfig;
	queueProvider?: IQueueProvider | undefined;
	logger: LoggerInterface;
	dependencies?: unknown;
	taskRegistry?: WorkerTaskRegistry | undefined;
	tracing?: TracingInterface | undefined;
}

export interface WorkerQueueOptions {
	queueProvider?: IQueueProvider | undefined;
	queueBaseUrl?: string | undefined;
	requestTimeoutMs?: number | undefined;
}

interface ResolvedWorkerFactoryOptions {
	queue: WorkerQueueOptions;
	runtime: WorkerRuntimeConfig;
	logger: LoggerInterface;
	dependencies?: unknown;
	taskRegistry?: WorkerTaskRegistry | undefined;
	tracing?: TracingInterface | undefined;
}

export interface WorkerResult {
	start: () => Promise<void>;
	shutdown: () => Promise<void>;
	processTask: (job: LeasedQueueJob) => Promise<void>;
	getRunner: () => WorkerRunner;
	getWorkerService: () => IWorkerService;
	registerTask: <TPayload = Record<string, unknown>>(name: string, handler: WorkerTaskHandler<TPayload>) => void;
	registerTasks: (tasks: Record<string, WorkerTaskHandler>) => void;
}

type WorkerFactoryOptions = CreateWorkerOptions | CreateWorkerLegacyOptions;

function isLegacyCreateWorkerOptions(options: WorkerFactoryOptions): options is CreateWorkerLegacyOptions {
	return 'config' in options;
}

function resolveLegacyQueueOptions(config: WorkerQueueConfig, queueProvider?: IQueueProvider): WorkerQueueOptions {
	return {
		queueProvider,
		queueBaseUrl: config.queueBaseUrl,
		requestTimeoutMs: config.requestTimeoutMs,
	};
}

function resolveWorkerFactoryOptions(options: WorkerFactoryOptions): ResolvedWorkerFactoryOptions {
	if (isLegacyCreateWorkerOptions(options)) {
		return {
			queue: resolveLegacyQueueOptions(options.config, options.queueProvider),
			runtime: {
				workerId: options.config.workerId,
				taskTypes: options.config.taskTypes,
				concurrency: options.config.concurrency,
			},
			logger: options.logger,
			dependencies: options.dependencies,
			taskRegistry: options.taskRegistry,
			tracing: options.tracing,
		};
	}
	return {
		queue: options.queue,
		runtime: options.runtime ?? {},
		logger: options.logger,
		dependencies: options.dependencies,
		taskRegistry: options.taskRegistry,
		tracing: options.tracing,
	};
}

function assertTaskRegistryMutable(runner: WorkerRunner | null): void {
	if (runner?.isRunning()) {
		throw new Error('Cannot register tasks after worker start. Register tasks before starting the worker.');
	}
}

export function createWorker(options: WorkerFactoryOptions): WorkerResult {
	const resolvedOptions = resolveWorkerFactoryOptions(options);
	const {queue, runtime, logger, dependencies, taskRegistry: providedRegistry, tracing} = resolvedOptions;
	if (dependencies !== undefined) {
		setWorkerDependencies(dependencies);
	}
	const taskRegistry = providedRegistry ?? new WorkerTaskRegistry();
	let runner: WorkerRunner | null = null;
	let workerService: WorkerService | null = null;
	function ensureRunner(): WorkerRunner {
		if (!runner) {
			runner = new WorkerRunner({
				tasks: taskRegistry.getTasks(),
				queueBaseUrl: queue.queueBaseUrl,
				queueProvider: queue.queueProvider,
				logger,
				workerId: runtime.workerId,
				taskTypes: runtime.taskTypes,
				concurrency: runtime.concurrency,
				tracing,
				requestTimeoutMs: queue.requestTimeoutMs,
			});
		}
		return runner;
	}
	function ensureWorkerService(): WorkerService {
		if (!workerService) {
			workerService = new WorkerService({
				queueBaseUrl: queue.queueBaseUrl,
				queueProvider: queue.queueProvider,
				logger,
				tracing,
				timeoutMs: queue.requestTimeoutMs,
			});
		}
		return workerService;
	}
	return {
		async start() {
			const r = ensureRunner();
			await r.start();
		},
		async shutdown() {
			if (runner) {
				await runner.stop();
			}
		},
		async processTask(job: LeasedQueueJob) {
			const r = ensureRunner();
			await r.processJob(job);
		},
		getRunner() {
			return ensureRunner();
		},
		getWorkerService() {
			return ensureWorkerService();
		},
		registerTask<TPayload = Record<string, unknown>>(name: string, handler: WorkerTaskHandler<TPayload>) {
			assertTaskRegistryMutable(runner);
			taskRegistry.register(name, handler);
			runner = null;
		},
		registerTasks(tasks: Record<string, WorkerTaskHandler>) {
			assertTaskRegistryMutable(runner);
			taskRegistry.registerAll(tasks);
			runner = null;
		},
	};
}
