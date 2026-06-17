// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CaptchaProviderType} from '@pkgs/captcha/src/ICaptchaProvider';
import {HttpCaptchaProvider} from '@pkgs/captcha/src/providers/HttpCaptchaProvider';

export class HcaptchaProvider extends HttpCaptchaProvider {
	readonly type: CaptchaProviderType = 'hcaptcha';
	protected readonly verifyUrl = 'https://api.hcaptcha.com/siteverify';
	protected readonly providerName = 'hCaptcha';
}
