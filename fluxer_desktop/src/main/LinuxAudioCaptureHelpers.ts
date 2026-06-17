// SPDX-License-Identifier: AGPL-3.0-or-later

const MAX_WINDOW_SOURCE_ID_LENGTH = 256;
const MAX_WINDOW_TOKEN_LENGTH = 128;

export function isX11SessionEnv(env: NodeJS.ProcessEnv): boolean {
	const sessionType = (env.XDG_SESSION_TYPE ?? '').toLowerCase();
	if (sessionType === 'x11') return true;
	if (sessionType === 'wayland') return false;
	return Boolean(env.DISPLAY);
}

export function isWaylandSessionEnv(env: NodeJS.ProcessEnv): boolean {
	const sessionType = (env.XDG_SESSION_TYPE ?? '').toLowerCase();
	if (sessionType === 'wayland') return true;
	if (sessionType === 'x11') return false;
	return Boolean(env.WAYLAND_DISPLAY);
}

export function parseWindowSourceToken(sourceId: unknown): string | null {
	if (typeof sourceId !== 'string' || sourceId.length > MAX_WINDOW_SOURCE_ID_LENGTH) return null;
	const match = /^window:([^:]+):(?:0|1)$/.exec(sourceId);
	const token = match?.[1] ?? null;
	if (!token || token.length > MAX_WINDOW_TOKEN_LENGTH) return null;
	return token;
}

export function isX11WindowToken(token: string): boolean {
	return /^(?:0x[0-9a-fA-F]+|[0-9]+)$/.test(token);
}

export function isDBusObjectPathSegment(token: string): boolean {
	return /^[A-Za-z0-9_]+$/.test(token);
}

function _gnomeShellPidExpression(token: string): string {
	const tokenLiteral = JSON.stringify(token);
	return `global.get_window_actors().map(a=>a.meta_window).filter(w=>w.get_id&&w.get_id().toString()===${tokenLiteral}).map(w=>w.get_pid())[0]`;
}
