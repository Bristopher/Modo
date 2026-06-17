// SPDX-License-Identifier: AGPL-3.0-or-later

export function assertPositiveFiniteNumber(value: number, fieldName: string): void {
	if (!Number.isFinite(value) || value <= 0) {
		throw new Error(`${fieldName} must be a positive finite number`);
	}
}

export function assertNonEmptyString(value: string, fieldName: string): void {
	if (value.length === 0) {
		throw new Error(`${fieldName} must be a non-empty string`);
	}
}
