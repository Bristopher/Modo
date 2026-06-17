// SPDX-License-Identifier: AGPL-3.0-or-later

export type InputEvent =
	| {
			type: 'keydown' | 'keyup';
			keycode: number;
			keyName: string;
			ctrlKey: boolean;
			altKey: boolean;
			shiftKey: boolean;
			metaKey: boolean;
	  }
	| {
			type: 'mousedown' | 'mouseup';
			button: number;
			ctrlKey: boolean;
			altKey: boolean;
			shiftKey: boolean;
			metaKey: boolean;
			x?: number;
			y?: number;
	  }
	| {
			type: 'mousemove';
			x: number;
			y: number;
			ctrlKey: boolean;
			altKey: boolean;
			shiftKey: boolean;
			metaKey: boolean;
	  }
	| {
			type: 'wheel';
			x?: number;
			y?: number;
			deltaX: number;
			deltaY: number;
			ctrlKey: boolean;
			altKey: boolean;
			shiftKey: boolean;
			metaKey: boolean;
	  };

export declare class InputHook {
	constructor(callback: (event: InputEvent) => void);

	start(): void;

	stop(): void;

	readonly droppedEvents: number;
	readonly reinstallCount: number;
}

export declare function isAvailable(): boolean;

export declare const loadError: Error | null;
