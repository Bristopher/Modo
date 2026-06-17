// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHelpers} from '@pkgs/worker/src/contracts/WorkerTask';
import {z} from 'zod';
import {ScheduledMessageExecutor, type SendScheduledMessageParams} from '../executors/ScheduledMessageExecutor';
import {getWorkerDependencies} from '../WorkerContext';

const PayloadSchema = z.object({
	userId: z.string(),
	scheduledMessageId: z.string(),
	expectedScheduledAt: z.string(),
});

export async function sendScheduledMessage(payload: unknown, helpers: WorkerTaskHelpers): Promise<void> {
	const validated = PayloadSchema.parse(payload) as SendScheduledMessageParams;
	const deps = getWorkerDependencies();
	const executor = new ScheduledMessageExecutor(deps, helpers.logger);
	const result = await executor.execute(validated);
	return result;
}
