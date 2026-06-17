// SPDX-License-Identifier: AGPL-3.0-or-later

type TestHarnessResetHandler = () => Promise<void>;

const registeredHandler: TestHarnessResetHandler | null = null;

export async function resetTestHarnessState(): Promise<void> {
	if (!registeredHandler) {
		throw new Error('Test harness reset handler not registered');
	}
	await registeredHandler();
}
