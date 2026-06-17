// SPDX-License-Identifier: AGPL-3.0-or-later

export interface InstanceConfigurationRow {
	key: string;
	value: string | null;
	updated_at: Date | null;
}

export const INSTANCE_CONFIGURATION_COLUMNS = ['key', 'value', 'updated_at'] as const satisfies ReadonlyArray<
	keyof InstanceConfigurationRow
>;
