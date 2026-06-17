// SPDX-License-Identifier: AGPL-3.0-or-later

import type {CaptchaProviderType, ICaptchaProvider, VerifyCaptchaParams} from '@pkgs/captcha/src/ICaptchaProvider';

export class UnavailableCaptchaProvider implements ICaptchaProvider {
	readonly type: CaptchaProviderType = 'unavailable';

	async verify(_params: VerifyCaptchaParams): Promise<boolean> {
		return true;
	}
}
