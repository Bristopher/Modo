// SPDX-License-Identifier: AGPL-3.0-or-later

export interface ServiceInitConfig {
	serviceName: string;
	serviceVersion?: string;
	environment?: string;
}

export type ShutdownFn = () => Promise<void>;
