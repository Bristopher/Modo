// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GatewayRolloutConfig} from '@fluxer/schema/src/domains/admin/GatewayRolloutSchemas';
import type {INatsConnectionManager} from '@pkgs/nats/src/INatsConnectionManager';
import {StringCodec} from 'nats';

export const GATEWAY_ROLLOUT_CONFIG_NATS_SUBJECT = 'config.gateway.rollout';

interface GatewayRolloutConfigNatsMessage {
	type: 'gateway_rollout_config';
	config: GatewayRolloutConfig;
}

export class GatewayRolloutConfigPublisher {
	private readonly codec = StringCodec();

	constructor(private readonly connectionManager: INatsConnectionManager) {}

	async publish(config: GatewayRolloutConfig): Promise<void> {
		if (this.connectionManager.isClosed()) {
			await this.connectionManager.connect();
		}
		const connection = this.connectionManager.getConnection();
		const message: GatewayRolloutConfigNatsMessage = {
			type: 'gateway_rollout_config',
			config,
		};
		connection.publish(GATEWAY_ROLLOUT_CONFIG_NATS_SUBJECT, this.codec.encode(JSON.stringify(message)));
		await connection.flush();
	}
}
