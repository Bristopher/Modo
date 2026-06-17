// SPDX-License-Identifier: AGPL-3.0-or-later

import {NatsConnectionManager} from '@pkgs/nats/src/NatsConnectionManager';
import type {JetStreamClient, JetStreamManager} from 'nats';

export class JetStreamConnectionManager extends NatsConnectionManager {
	getJetStreamClient(): JetStreamClient {
		return this.getConnection().jetstream();
	}

	async getJetStreamManager(): Promise<JetStreamManager> {
		return this.getConnection().jetstreamManager();
	}
}
