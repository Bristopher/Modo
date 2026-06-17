// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {z} from 'zod';
import {createChannelID, createGuildID, createUserID} from '../../BrandedTypes';
import {UserMessageDeletionService} from '../../channel/services/message/UserMessageDeletionService';
import {Logger} from '../../Logger';
import {getWorkerDependencies} from '../WorkerContext';

const PayloadSchema = z.object({
	userId: z.string(),
	channelIds: z.array(z.string()).optional(),
	guildId: z.string().optional(),
});
const bulkDeleteUserMessagesScoped: WorkerTaskHandler = async (payload, helpers) => {
	const validated = PayloadSchema.parse(payload);
	helpers.logger.debug({payload: validated}, 'Processing bulkDeleteUserMessagesScoped task');
	const userId = createUserID(BigInt(validated.userId));
	const {channelRepository, gatewayService, storageService, purgeQueue} = getWorkerDependencies();
	const deletionService = new UserMessageDeletionService({
		channelRepository,
		gatewayService,
		storageService,
		purgeQueue,
	});
	const totalDeleted = await deletionService.deleteUserMessagesInScope(
		userId,
		{
			channelIds: validated.channelIds?.map((id) => createChannelID(BigInt(id))),
			guildId: validated.guildId ? createGuildID(BigInt(validated.guildId)) : undefined,
		},
		{
			onProgress: (deleted) => helpers.logger.debug(`Deleted ${deleted} messages so far`),
		},
	);
	Logger.debug({userId: userId.toString(), totalDeleted}, 'Scoped bulk message deletion completed');
};

export default bulkDeleteUserMessagesScoped;
