// SPDX-License-Identifier: AGPL-3.0-or-later

export function derivePlusAddressBase(email: string | null | undefined): string | null {
	if (!email) {
		return null;
	}
	const trimmed = email.trim().toLowerCase();
	const atIndex = trimmed.lastIndexOf('@');
	if (atIndex <= 0 || atIndex === trimmed.length - 1) {
		return null;
	}
	const localPart = trimmed.slice(0, atIndex);
	const domain = trimmed.slice(atIndex + 1);
	const plusIndex = localPart.indexOf('+');
	if (plusIndex <= 0) {
		return null;
	}
	return `${localPart.slice(0, plusIndex)}@${domain}`;
}
