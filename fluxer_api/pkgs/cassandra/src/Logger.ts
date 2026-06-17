// SPDX-License-Identifier: AGPL-3.0-or-later

export interface Logger {
	info(context: Record<string, unknown>, message: string): void;
	debug(context: Record<string, unknown>, message: string): void;
	error(context: Record<string, unknown>, message: string): void;
}

export const NoopLogger: Logger = {
	info(): void {},
	debug(): void {},
	error(): void {},
};
