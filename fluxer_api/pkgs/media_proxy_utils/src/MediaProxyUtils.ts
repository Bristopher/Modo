// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';
import {createExternalMediaProxyUrlBuilder} from '@pkgs/media_proxy_utils/src/ExternalMediaProxyUrlBuilder';

const BASE64_URL_REGEX = /=*$/;

export function createSignature(inputString: string, mediaProxySecretKey: string): string {
	const hmac = crypto.createHmac('sha256', mediaProxySecretKey);
	hmac.update(inputString);
	return hmac.digest('base64url').replace(BASE64_URL_REGEX, '');
}

export function verifySignature(proxyUrlPath: string, providedSignature: string, mediaProxySecretKey: string): boolean {
	const expectedSignature = createSignature(proxyUrlPath, mediaProxySecretKey);
	const expectedBuffer = Buffer.from(expectedSignature);
	const providedBuffer = Buffer.from(providedSignature);
	if (expectedBuffer.length !== providedBuffer.length) {
		return false;
	}
	return crypto.timingSafeEqual(expectedBuffer, providedBuffer);
}

export interface ExternalMediaProxyURLOptions {
	inputURL: string;
	mediaProxyEndpoint: string;
	mediaProxySecretKey: string;
}

export function getExternalMediaProxyURL(options: ExternalMediaProxyURLOptions): string {
	const builder = createExternalMediaProxyUrlBuilder({
		mediaProxyEndpoint: options.mediaProxyEndpoint,
		mediaProxySecretKey: options.mediaProxySecretKey,
	});
	return builder.buildExternalMediaProxyUrl(options.inputURL);
}
