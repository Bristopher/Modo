// SPDX-License-Identifier: AGPL-3.0-or-later

import type {TracingInterface} from '@pkgs/worker/src/contracts/WorkerTypes';
import {HttpWorkerQueue} from '@pkgs/worker/src/providers/HttpWorkerQueue';
import type {IQueueProvider} from '@pkgs/worker/src/providers/IQueueProvider';

export interface QueueProviderFactoryOptions {
	queueProvider?: IQueueProvider | undefined;
	queueBaseUrl?: string | undefined;
	timeoutMs?: number | undefined;
	tracing?: TracingInterface | undefined;
}

export function createQueueProvider(options: QueueProviderFactoryOptions): IQueueProvider {
	if (options.queueProvider) {
		return options.queueProvider;
	}
	if (!options.queueBaseUrl) {
		throw new Error('Queue provider requires either queueProvider or queueBaseUrl');
	}
	return new HttpWorkerQueue({
		baseUrl: options.queueBaseUrl,
		timeoutMs: options.timeoutMs,
		tracing: options.tracing,
	});
}
