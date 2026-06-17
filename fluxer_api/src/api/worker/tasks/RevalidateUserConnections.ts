// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {z} from 'zod';
import {createUserID} from '../../BrandedTypes';
import {mapConnectionToResponse} from '../../connection/ConnectionMappers';
import {Logger} from '../../Logger';
import {getWorkerDependencies} from '../WorkerContext';

const PayloadSchema = z.object({
	userId: z.string(),
});
const revalidateUserConnections: WorkerTaskHandler = async (payload, helpers) => {
	const validated = PayloadSchema.parse(payload);
	helpers.logger.debug({payload: validated}, 'Processing revalidateUserConnections task');
	const userId = createUserID(BigInt(validated.userId));
	const {connectionRepository, connectionService, gatewayService} = getWorkerDependencies();
	const connections = await connectionRepository.findByUserId(userId);
	const verifiedConnections = connections.filter((conn) => conn.verified);
	if (verifiedConnections.length === 0) {
		helpers.logger.debug({userId: userId.toString()}, 'No verified connections to revalidate');
		return;
	}
	let hasChanges = false;
	for (const connection of verifiedConnections) {
		try {
			const {isValid, updateParams} = await connectionService.revalidateConnection(connection);
			if (updateParams) {
				await connectionRepository.update(userId, connection.connection_type, connection.connection_id, updateParams);
				hasChanges = true;
				if (!isValid) {
					Logger.info(
						{
							userId: userId.toString(),
							connectionId: connection.connection_id,
							connectionType: connection.connection_type,
						},
						'Connection verification failed, marked as unverified',
					);
				}
			}
		} catch (error) {
			Logger.error(
				{
					error,
					userId: userId.toString(),
					connectionId: connection.connection_id,
					connectionType: connection.connection_type,
				},
				'Failed to revalidate connection',
			);
		}
	}
	if (hasChanges) {
		const updatedConnections = await connectionRepository.findByUserId(userId);
		await gatewayService.dispatchPresence({
			userId,
			event: 'USER_CONNECTIONS_UPDATE',
			data: {connections: updatedConnections.map(mapConnectionToResponse)},
		});
		Logger.info({userId: userId.toString()}, 'Dispatched USER_CONNECTIONS_UPDATE event');
	}
	helpers.logger.debug(
		{userId: userId.toString(), checked: verifiedConnections.length, hasChanges},
		'Completed connection revalidation',
	);
};

export default revalidateUserConnections;
