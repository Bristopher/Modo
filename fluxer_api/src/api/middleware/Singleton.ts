// SPDX-License-Identifier: AGPL-3.0-or-later

export function singleton<T>(factory: () => T): () => T {
	let instance: T | undefined;
	return () => {
		if (instance === undefined) {
			instance = factory();
		}
		return instance;
	};
}
