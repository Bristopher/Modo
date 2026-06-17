// SPDX-License-Identifier: AGPL-3.0-or-later

export interface GlobalKeyHookBackend {
	start(): Promise<boolean> | boolean;
	stop(): void;
}

export class GlobalKeyHookLifecycle {
	private readonly acquisitionsByOwner = new Map<number, number>();
	private running = false;
	private queue: Promise<unknown> = Promise.resolve();

	constructor(private readonly backend: GlobalKeyHookBackend) {}

	isRunning(): boolean {
		return this.running;
	}

	acquisitionCount(): number {
		let total = 0;
		for (const count of this.acquisitionsByOwner.values()) {
			total += count;
		}
		return total;
	}

	acquire(ownerId: number): Promise<boolean> {
		return this.enqueue(async () => {
			if (!this.running) {
				let started = false;
				try {
					started = await this.backend.start();
				} catch {
					started = false;
				}
				if (!started) return false;
				this.running = true;
			}
			this.acquisitionsByOwner.set(ownerId, (this.acquisitionsByOwner.get(ownerId) ?? 0) + 1);
			return true;
		});
	}

	release(ownerId: number): Promise<void> {
		return this.enqueue(async () => {
			const count = this.acquisitionsByOwner.get(ownerId) ?? 0;
			if (count === 0) return;
			if (count === 1) {
				this.acquisitionsByOwner.delete(ownerId);
			} else {
				this.acquisitionsByOwner.set(ownerId, count - 1);
			}
			this.stopIfIdle();
		});
	}

	releaseAllForOwner(ownerId: number): Promise<void> {
		return this.enqueue(async () => {
			if (!this.acquisitionsByOwner.delete(ownerId)) return;
			this.stopIfIdle();
		});
	}

	forceStop(): Promise<void> {
		return this.enqueue(async () => {
			this.acquisitionsByOwner.clear();
			this.stopIfIdle();
		});
	}

	private stopIfIdle(): void {
		if (!this.running) return;
		if (this.acquisitionCount() > 0) return;
		this.running = false;
		try {
			this.backend.stop();
		} catch {}
	}

	private enqueue<T>(task: () => Promise<T>): Promise<T> {
		const run = this.queue.then(task);
		this.queue = run.then(
			() => undefined,
			() => undefined,
		);
		return run;
	}
}
