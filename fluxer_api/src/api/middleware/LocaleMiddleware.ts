// SPDX-License-Identifier: AGPL-3.0-or-later

import {Locales} from '@fluxer/constants/src/Locales';
import {parseAcceptLanguage} from '@pkgs/locale/src/LocaleService';
import {createMiddleware} from 'hono/factory';
import type {HonoEnv} from '../types/HonoEnv';

export const LocaleMiddleware = createMiddleware<HonoEnv>(async (ctx, next) => {
	const acceptLanguage = ctx.req.header('accept-language');
	const headerLocale = parseAcceptLanguage(acceptLanguage);
	const user = ctx.get('user');
	const locale = user?.locale ?? headerLocale ?? Locales.EN_US;
	ctx.set('requestLocale', locale);
	return next();
});
