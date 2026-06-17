// SPDX-License-Identifier: AGPL-3.0-or-later

import {
	clearWorkerDependencies as clearWorkerDependenciesBase,
	getWorkerDependencies as getWorkerDependenciesBase,
	setWorkerDependencies as setWorkerDependenciesBase,
} from '@pkgs/worker/src/context/WorkerContext';
import type {WorkerDependencies} from './WorkerDependencies';

export function setWorkerDependencies(dependencies: WorkerDependencies): void {
	setWorkerDependenciesBase(dependencies);
}

export function setWorkerDependenciesForTest(dependencies: Partial<WorkerDependencies>): void {
	setWorkerDependenciesBase(dependencies);
}

export function getWorkerDependencies(): WorkerDependencies {
	return getWorkerDependenciesBase<WorkerDependencies>();
}

export function clearWorkerDependencies(): void {
	clearWorkerDependenciesBase();
}
