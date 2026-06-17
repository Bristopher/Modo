// SPDX-License-Identifier: AGPL-3.0-or-later

interface KVRequiredErrorOptions {
	serviceName: string;
	configPath: string;
}

export function throwKVRequiredError(options: KVRequiredErrorOptions): never {
	const {serviceName, configPath} = options;
	throw new Error(
		`${serviceName} requires KV-backed rate limiting. ${configPath} is not set. ` +
			`internal.kv must be configured for distributed rate limiting.`,
	);
}
