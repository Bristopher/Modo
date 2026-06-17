// SPDX-License-Identifier: AGPL-3.0-or-later

import type {Context} from 'hono';
import {getCookie, setCookie} from 'hono/cookie';
import {seconds} from 'itty-time';
import {Config} from '../Config';
import type {HonoEnv} from '../types/HonoEnv';

const SUDO_COOKIE_PREFIX = '__flx_sudo';
const SUDO_COOKIE_MAX_AGE = seconds('5 minutes');

function getCookieDomain(): string {
	const domain = Config.cookie.domain;
	if (domain) {
		return domain;
	}
	try {
		const url = new URL(Config.endpoints.webApp);
		const hostname = url.hostname;
		const parts = hostname.split('.');
		if (parts.length >= 2) {
			return `.${parts.slice(-2).join('.')}`;
		} else {
			return hostname;
		}
	} catch {
		return '';
	}
}

function getSudoCookieOptions() {
	return {
		httpOnly: true,
		secure: Config.cookie.secure,
		sameSite: 'Strict' as const,
		domain: getCookieDomain(),
		path: '/',
		maxAge: SUDO_COOKIE_MAX_AGE,
	};
}

function sudoCookieName(userId?: string | number): string {
	if (userId === undefined || userId === null) {
		return SUDO_COOKIE_PREFIX;
	}
	return `${SUDO_COOKIE_PREFIX}_${userId}`;
}

export function setSudoCookie(ctx: Context<HonoEnv>, token: string, userId?: string | number): void {
	const cookieName = sudoCookieName(userId);
	const options = getSudoCookieOptions();
	setCookie(ctx, cookieName, token, options);
}

export function getSudoCookie(ctx: Context<HonoEnv>, userId?: string | number): string | undefined {
	const cookieName = sudoCookieName(userId);
	return getCookie(ctx, cookieName);
}
