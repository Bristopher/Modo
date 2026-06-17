// SPDX-License-Identifier: AGPL-3.0-or-later

export interface ResolveWindowPidSpec {
	backend: 'gnome-shell-eval';
	token: string;
}

export declare function resolveWindowPid(spec: ResolveWindowPidSpec): Promise<number | null>;

export declare function resolveKwinWindowPid(token: string): Promise<number | null>;

export declare function resolveX11WindowPid(token: string): Promise<number | null>;

export type FileChooserFilterKind = 0 | 1;

export interface FileChooserFilterRule {
	kind: FileChooserFilterKind;
	pattern: string;
}

export interface FileChooserFilter {
	name: string;
	rules: ReadonlyArray<FileChooserFilterRule>;
}

export interface FileChooserOptions {
	parentWindow?: string;
	title?: string;
	acceptLabel?: string;
	modal?: boolean;
	multiple?: boolean;
	directory?: boolean;
	currentFolder?: string;
	currentName?: string;
	currentFile?: string;
	filters?: ReadonlyArray<FileChooserFilter>;
	currentFilter?: FileChooserFilter;
}

export interface FileChooserResult {
	cancelled: boolean;
	uris: Array<string>;
}

export declare function openFile(options: FileChooserOptions): Promise<FileChooserResult>;

export declare function saveFile(options: FileChooserOptions): Promise<FileChooserResult>;

export interface BackgroundOptions {
	reason?: string;
	autostart?: boolean;
	commandline?: ReadonlyArray<string>;
	dbusActivatable?: boolean;
}

export interface BackgroundResult {
	response: number;
	cancelled: boolean;
	background: boolean;
	autostart: boolean;
}

export declare function requestBackground(options: BackgroundOptions): Promise<BackgroundResult>;

export interface GlobalShortcutEntry {
	id: string;
	description: string;
	preferredTrigger?: string;
}

export interface BoundGlobalShortcut {
	id: string;
	description?: string;
	triggerDescription?: string;
}

export type GlobalShortcutPortalAction = 'listed' | 'bound' | 'cancelled';

export interface GlobalShortcutsConfigureResult {
	action: GlobalShortcutPortalAction;
	shortcuts: Array<BoundGlobalShortcut>;
}

export type GlobalShortcutPortalEvent =
	| {type: 'activated'; id: string}
	| {type: 'deactivated'; id: string}
	| {type: 'shortcuts-changed'; shortcuts: Array<BoundGlobalShortcut>}
	| {type: 'closed'};

export declare class GlobalShortcutsPortal {
	constructor(onEvent: (event: GlobalShortcutPortalEvent) => void, appId?: string | null);

	configure(entries: ReadonlyArray<GlobalShortcutEntry>): Promise<GlobalShortcutsConfigureResult>;

	close(): void;
}

export declare function isAvailable(): boolean;

export declare function getPortalVersion(): number | null;

export declare function readColorScheme(): 'no-preference' | 'prefer-dark' | 'prefer-light';

export declare function readContrast(): 'no-preference' | 'high';

export declare function readAccentColor(): {r: number; g: number; b: number} | null;

export interface SettingsChangeEvent {
	namespace: string;
	key: string;
	uint32?: number;
	accent?: {r: number; g: number; b: number};
}

export declare class Settings {
	constructor(onChange: (event: SettingsChangeEvent) => void);

	close(): void;
}

export declare const loadError: Error | null;
