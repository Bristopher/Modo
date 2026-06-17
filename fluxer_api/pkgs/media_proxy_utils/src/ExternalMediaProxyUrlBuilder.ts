// SPDX-License-Identifier: AGPL-3.0-or-later

import {buildExternalMediaProxyPath} from '@pkgs/media_proxy_utils/src/ExternalMediaProxyPathCodec';
import {createMediaProxySigner, type IMediaProxySigner} from '@pkgs/media_proxy_utils/src/MediaProxySigner';

export interface ExternalMediaProxyUrlBuilderOptions {
	mediaProxyEndpoint: string;
	mediaProxySecretKey: string;
}

export interface IExternalMediaProxyUrlBuilder {
	buildExternalMediaProxyUrl(inputUrl: string): string;
}

interface InternalExternalMediaProxyUrlBuilderOptions {
	mediaProxyEndpoint: string;
	mediaProxySigner: IMediaProxySigner;
}

function normalizeMediaProxyEndpoint(mediaProxyEndpoint: string): string {
	return mediaProxyEndpoint.replace(/\/+$/u, '');
}

class ExternalMediaProxyUrlBuilder implements IExternalMediaProxyUrlBuilder {
	private readonly mediaProxyEndpoint: string;
	private readonly mediaProxySigner: IMediaProxySigner;

	constructor(options: InternalExternalMediaProxyUrlBuilderOptions) {
		this.mediaProxyEndpoint = options.mediaProxyEndpoint;
		this.mediaProxySigner = options.mediaProxySigner;
	}

	buildExternalMediaProxyUrl(inputUrl: string): string {
		const proxyUrlPath = buildExternalMediaProxyPath(inputUrl);
		const signature = this.mediaProxySigner.createSignature(proxyUrlPath);
		return `${this.mediaProxyEndpoint}/external/${signature}/${proxyUrlPath}`;
	}
}

export function createExternalMediaProxyUrlBuilder(
	options: ExternalMediaProxyUrlBuilderOptions,
): IExternalMediaProxyUrlBuilder {
	const mediaProxySigner = createMediaProxySigner({
		mediaProxySecretKey: options.mediaProxySecretKey,
	});
	return new ExternalMediaProxyUrlBuilder({
		mediaProxyEndpoint: normalizeMediaProxyEndpoint(options.mediaProxyEndpoint),
		mediaProxySigner,
	});
}
