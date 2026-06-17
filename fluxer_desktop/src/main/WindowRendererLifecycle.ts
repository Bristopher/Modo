// SPDX-License-Identifier: AGPL-3.0-or-later

export interface MainWindowRendererGoneContext {
	platform: NodeJS.Platform;
	isQuitting: boolean;
	isMainWindowHidden: boolean;
	closeToTrayEnabled: boolean;
	reloadedRecently: boolean;
}

export type MainWindowRendererGoneAction = 'reload' | 'defer-reload' | 'ignore' | 'quit';

function isRecoverableHiddenCloseToTrayReason(details: Electron.RenderProcessGoneDetails): boolean {
	return details.reason === 'clean-exit' || details.reason === 'killed';
}

function shouldKeepHiddenCloseToTrayWindowAlive(context: MainWindowRendererGoneContext): boolean {
	return (
		context.platform === 'darwin' && !context.isQuitting && context.isMainWindowHidden && context.closeToTrayEnabled
	);
}

export function getMainWindowRendererGoneAction(
	details: Electron.RenderProcessGoneDetails,
	context: MainWindowRendererGoneContext,
): MainWindowRendererGoneAction {
	if (context.isQuitting) {
		return 'ignore';
	}
	if (shouldKeepHiddenCloseToTrayWindowAlive(context)) {
		if (context.reloadedRecently) {
			return 'defer-reload';
		}
		if (isRecoverableHiddenCloseToTrayReason(details)) {
			return 'reload';
		}
	}
	if (details.reason === 'killed' || context.reloadedRecently) {
		return 'quit';
	}
	if (details.reason === 'clean-exit') {
		return 'ignore';
	}
	return 'reload';
}
