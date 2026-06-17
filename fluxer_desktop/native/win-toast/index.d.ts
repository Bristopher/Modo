// SPDX-License-Identifier: AGPL-3.0-or-later

export interface ToastAction {
	label: string;
	args: string;
	activationType?: 'foreground' | 'background' | 'protocol';
	imageUri?: string;
	hintInputId?: string;
}

export interface ToastBindingText {
	text: string;
	hintMaxLines?: number;
	hint?: 'attribution';
}

export interface ToastInputBox {
	id: string;
	type: 'text' | 'selection';
	placeholder?: string;
	title?: string;
	options?: ReadonlyArray<{id: string; content: string}>;
}

export interface ToastImage {
	uri: string;
	placement?: 'hero' | 'appLogoOverride';
	hintCrop?: 'circle' | 'none';
	alt?: string;
}

export interface ToastNotifyOptions {
	aumid: string;
	tag?: string;
	group?: string;
	expirationTime?: string;
	scenario?: 'default' | 'urgent' | 'reminder' | 'incomingCall' | 'alarm';
	audio?: 'default' | 'silent' | {silent?: boolean; loop?: boolean; src?: string};
	lines: ReadonlyArray<ToastBindingText>;
	images?: ReadonlyArray<ToastImage>;
	inputs?: ReadonlyArray<ToastInputBox>;
	actions?: ReadonlyArray<ToastAction>;
}

export interface ToastSupport {
	supported: boolean;
	reason?: string;
}

export declare function isSupported(): ToastSupport;

export declare function notify(opts: ToastNotifyOptions): Promise<void>;

export declare function dismiss(opts: {aumid: string; tag: string; group?: string}): Promise<void>;

export declare function clear(opts: {aumid: string}): Promise<void>;

export declare const loadError: Error | null;
