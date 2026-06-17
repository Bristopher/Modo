// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ServiceInitConfig, ShutdownFn} from '@pkgs/initialization/src/ServiceInitializationTypes';

let initialized = false;

export async function initializeService(_config: ServiceInitConfig): Promise<void> {
	if (initialized) {
		return;
	}
	initialized = true;
}

export async function shutdownService(): Promise<void> {
	if (!initialized) {
		return;
	}
	initialized = false;
}

export function isServiceInitialized(): boolean {
	return initialized;
}

export function startServiceInitialization(config: ServiceInitConfig): ShutdownFn {
	initializeService(config).catch((err: unknown) => {
		process.stderr.write(`[instrument] Failed to initialize instrumentation: ${err}\n`);
	});
	return shutdownService;
}
