// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';

const BASE64_URL_PADDING_REGEX = /=*$/;

export interface MediaProxySignerOptions {
	mediaProxySecretKey: string;
}

export interface IMediaProxySigner {
	createSignature(proxyUrlPath: string): string;
	verifySignature(proxyUrlPath: string, providedSignature: string): boolean;
}

class MediaProxySigner implements IMediaProxySigner {
	private readonly mediaProxySecretKey: string;

	constructor(options: MediaProxySignerOptions) {
		this.mediaProxySecretKey = options.mediaProxySecretKey;
	}

	createSignature(proxyUrlPath: string): string {
		const hmac = crypto.createHmac('sha256', this.mediaProxySecretKey);
		hmac.update(proxyUrlPath);
		return hmac.digest('base64url').replace(BASE64_URL_PADDING_REGEX, '');
	}

	verifySignature(proxyUrlPath: string, providedSignature: string): boolean {
		const expectedSignature = this.createSignature(proxyUrlPath);
		const expectedBuffer = Buffer.from(expectedSignature, 'utf8');
		const providedBuffer = Buffer.from(providedSignature, 'utf8');
		if (expectedBuffer.length !== providedBuffer.length) {
			return false;
		}
		return crypto.timingSafeEqual(expectedBuffer, providedBuffer);
	}
}

export function createMediaProxySigner(options: MediaProxySignerOptions): IMediaProxySigner {
	return new MediaProxySigner(options);
}
