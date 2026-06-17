// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CaptchaProviderType} from '@pkgs/captcha/src/ICaptchaProvider';
import type {HttpCaptchaProviderOptions} from '@pkgs/captcha/src/providers/HttpCaptchaProvider';
import {HttpCaptchaProvider} from '@pkgs/captcha/src/providers/HttpCaptchaProvider';

const DEFAULT_MINIMUM_SCORE = 0.5;

interface RecaptchaVerifyResponse {
	success: boolean;
	'error-codes'?: Array<string>;
	score?: number;
}

export interface RecaptchaProviderOptions extends HttpCaptchaProviderOptions {
	minimumScore?: number;
}

export class RecaptchaProvider extends HttpCaptchaProvider {
	readonly type: CaptchaProviderType = 'recaptcha';
	protected readonly verifyUrl = 'https://www.google.com/recaptcha/api/siteverify';
	protected readonly providerName = 'reCAPTCHA';
	private readonly minimumScore: number;

	constructor(options: RecaptchaProviderOptions) {
		super(options);
		this.minimumScore = options.minimumScore ?? DEFAULT_MINIMUM_SCORE;
	}

	protected override validateResponse(data: RecaptchaVerifyResponse): boolean {
		if (data.score !== undefined && data.score < this.minimumScore) {
			this.logger?.warn(
				{score: data.score, minimumScore: this.minimumScore},
				'reCAPTCHA score below minimum threshold',
			);
			return false;
		}
		return true;
	}
}
