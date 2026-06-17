// SPDX-License-Identifier: AGPL-3.0-or-later

export function calculateDistance(lat1: number, lon1: number, lat2: number, lon2: number): number {
	const R = 6371;
	const dLat = ((lat2 - lat1) * Math.PI) / 180;
	const dLon = ((lon2 - lon1) * Math.PI) / 180;
	const a =
		Math.sin(dLat / 2) * Math.sin(dLat / 2) +
		Math.cos((lat1 * Math.PI) / 180) * Math.cos((lat2 * Math.PI) / 180) * Math.sin(dLon / 2) * Math.sin(dLon / 2);
	const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
	return R * c;
}

export function parseCoordinate(value?: string | null): number | null {
	if (value == null) {
		return null;
	}
	const parsed = Number.parseFloat(value);
	return Number.isFinite(parsed) ? parsed : null;
}
