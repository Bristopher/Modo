// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {UserActivityBuffer} from '../../user/services/UserActivityBuffer';
import {getWorkerDependencies} from '../WorkerContext';

const flushUserActivityBuffer: WorkerTaskHandler = async (_payload, helpers) => {
	const {kvClient} = getWorkerDependencies();
	const buffer = new UserActivityBuffer(kvClient);
	const stats = await buffer.drainAndFlush();
	if (stats.drained > 0) {
		helpers.logger.debug({...stats}, 'Flushed user activity buffer to Cassandra');
	}
};

export default flushUserActivityBuffer;
