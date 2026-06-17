// SPDX-License-Identifier: AGPL-3.0-or-later

declare const document: {
	getElementById(elementId: string): {classList: {remove(className: string): void}} | null;
};
