// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CaptchaProviderType} from '@pkgs/captcha/src/ICaptchaProvider';
import {HttpCaptchaProvider} from '@pkgs/captcha/src/providers/HttpCaptchaProvider';

export class TurnstileProvider extends HttpCaptchaProvider {
	readonly type: CaptchaProviderType = 'turnstile';
	protected readonly verifyUrl = 'https://challenges.cloudflare.com/turnstile/v0/siteverify';
	protected readonly providerName = 'Turnstile';
}
