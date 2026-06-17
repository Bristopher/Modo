// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {z} from 'zod';
import {createUserID} from '../../BrandedTypes';
import {Logger} from '../../Logger';
import {processUserDeletion} from '../../user/services/UserDeletionService';
import {getWorkerDependencies} from '../WorkerContext';

const PayloadSchema = z.object({
	userId: z.string(),
	deletionReasonCode: z.number(),
});
const userProcessPendingDeletion: WorkerTaskHandler = async (payload, helpers) => {
	const validated = PayloadSchema.parse(payload);
	helpers.logger.debug({payload: validated}, 'Processing userProcessPendingDeletion task');
	const userId = createUserID(BigInt(validated.userId));
	try {
		const deps = getWorkerDependencies();
		await processUserDeletion(userId, validated.deletionReasonCode, deps);
	} catch (error) {
		Logger.error({error, userId}, 'Failed to delete user account');
		throw error;
	}
};

export default userProcessPendingDeletion;
