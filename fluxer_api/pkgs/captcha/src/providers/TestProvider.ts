// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CaptchaProviderType, ICaptchaProvider, VerifyCaptchaParams} from '@pkgs/captcha/src/ICaptchaProvider';

export class TestCaptchaProvider implements ICaptchaProvider {
	readonly type: CaptchaProviderType = 'test';

	async verify(_params: VerifyCaptchaParams): Promise<boolean> {
		return true;
	}
}
