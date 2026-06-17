// SPDX-License-Identifier: AGPL-3.0-or-later

import type {INatsConnectionManager} from '@pkgs/nats/src/INatsConnectionManager';
import type {NatsConnectionOptions} from '@pkgs/nats/src/NatsConnectionOptions';
import {connect, type NatsConnection} from 'nats';

const DEFAULT_MAX_RECONNECT_ATTEMPTS = -1;
const DEFAULT_RECONNECT_TIME_WAIT_MS = 500;
const DEFAULT_CONNECT_TIMEOUT_MS = 5000;

export class NatsConnectionManager implements INatsConnectionManager {
	private connection: NatsConnection | null = null;
	private connectPromise: Promise<NatsConnection> | null = null;
	private readonly options: NatsConnectionOptions;

	constructor(options: NatsConnectionOptions) {
		this.options = options;
	}

	async connect(): Promise<void> {
		if (this.connection !== null && !this.connection.isClosed()) {
			return;
		}
		if (this.connectPromise !== null) {
			this.connection = await this.connectPromise;
			return;
		}
		const connectPromise = connect({
			servers: this.options.url,
			token: this.options.token || undefined,
			name: this.options.name,
			maxReconnectAttempts: this.options.maxReconnectAttempts ?? DEFAULT_MAX_RECONNECT_ATTEMPTS,
			reconnectTimeWait: this.options.reconnectTimeWaitMs ?? DEFAULT_RECONNECT_TIME_WAIT_MS,
			timeout: this.options.connectTimeoutMs ?? DEFAULT_CONNECT_TIMEOUT_MS,
		});
		this.connectPromise = connectPromise;
		try {
			this.connection = await connectPromise;
		} finally {
			if (this.connectPromise === connectPromise) {
				this.connectPromise = null;
			}
		}
	}

	getConnection(): NatsConnection {
		if (this.connection === null || this.connection.isClosed()) {
			throw new Error('NATS connection is not established. Call connect() first.');
		}
		return this.connection;
	}

	async drain(): Promise<void> {
		if (this.connectPromise !== null) {
			try {
				this.connection = await this.connectPromise;
			} catch {
				this.connectPromise = null;
				return;
			}
		}
		if (this.connection === null) {
			return;
		}
		if (!this.connection.isClosed()) {
			await this.connection.drain();
		}
		this.connection = null;
	}

	isClosed(): boolean {
		return this.connection === null || this.connection.isClosed();
	}
}
