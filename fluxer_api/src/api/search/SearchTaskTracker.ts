// SPDX-License-Identifier: AGPL-3.0-or-later

let trackingEnabled = false;

const pendingTasks = new Set<Promise<unknown>>();

export function enableSearchTaskTracking(): void {
	trackingEnabled = true;
}

export function trackSearchTask<T>(promise: Promise<T>): Promise<T> {
	if (!trackingEnabled) {
		return promise;
	}
	pendingTasks.add(promise);
	void promise.finally(() => {
		pendingTasks.delete(promise);
	});
	return promise;
}

export async function drainSearchTasks(options?: {timeoutMs?: number}): Promise<void> {
	if (!trackingEnabled) {
		return;
	}
	const timeoutMs = options?.timeoutMs ?? 30000;
	const deadline = Date.now() + timeoutMs;
	while (pendingTasks.size > 0) {
		const snapshot = Array.from(pendingTasks);
		const remainingMs = deadline - Date.now();
		if (remainingMs <= 0) {
			throw new Error(`Timed out waiting for pending search tasks (${pendingTasks.size} remaining)`);
		}
		await Promise.race([
			Promise.allSettled(snapshot),
			new Promise<void>((_resolve, reject) => {
				setTimeout(() => {
					reject(new Error(`Timed out waiting for pending search tasks (${pendingTasks.size} remaining)`));
				}, remainingMs);
			}),
		]);
	}
}
