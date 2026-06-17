// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';

export class WorkerTaskRegistry {
	private readonly tasks: Map<string, WorkerTaskHandler> = new Map();

	register<TPayload = Record<string, unknown>>(name: string, handler: WorkerTaskHandler<TPayload>): this {
		this.tasks.set(name, handler as WorkerTaskHandler);
		return this;
	}

	registerAll(tasks: Record<string, WorkerTaskHandler>): this {
		for (const [name, handler] of Object.entries(tasks)) {
			this.tasks.set(name, handler);
		}
		return this;
	}

	get(name: string): WorkerTaskHandler | undefined {
		return this.tasks.get(name);
	}

	has(name: string): boolean {
		return this.tasks.has(name);
	}

	getTaskNames(): Array<string> {
		return Array.from(this.tasks.keys());
	}

	getTasks(): Record<string, WorkerTaskHandler> {
		return Object.fromEntries(this.tasks);
	}

	get size(): number {
		return this.tasks.size;
	}
}

export function createTaskRegistry(): WorkerTaskRegistry {
	return new WorkerTaskRegistry();
}
