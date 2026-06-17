// SPDX-License-Identifier: AGPL-3.0-or-later

export interface CreateShortcutOptions {
	lnkPath: string;
	target: string;
	args?: string;
	appUserModelId?: string;
	toastActivatorClsid?: string;
	iconPath?: string;
	iconIndex?: number;
	workingDir?: string;
	description?: string;
}

export interface SetRunValueOptions {
	name: string;
	command: string;
}

export declare function createShortcut(opts: CreateShortcutOptions): Promise<void>;

export declare function setCurrentUserRunValue(opts: SetRunValueOptions): Promise<void>;

export declare function deleteCurrentUserRunValue(name: string): Promise<void>;

export declare function getCurrentUserRunValue(name: string): Promise<string | null>;

export declare function getUserNotificationState(): string;

export declare const loadError: Error | null;
