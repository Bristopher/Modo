// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageEmbedResponse} from '@fluxer/schema/src/domains/message/EmbedSchemas';
import type {MediaProxyNsfwMode} from './IMediaService';

export interface UnfurlResult {
	embeds: Array<MessageEmbedResponse>;
	cacheTtlSeconds: number | null;
}

export interface UnfurlOptions {
	signal?: AbortSignal;
	bypassCache?: boolean;
	cacheOnly?: boolean;
}

export abstract class IUnfurlerService {
	async unfurl(
		url: string,
		nsfwMode?: MediaProxyNsfwMode,
		options: UnfurlOptions = {},
	): Promise<Array<MessageEmbedResponse>> {
		return (await this.unfurlWithCachePolicy(url, nsfwMode, options)).embeds;
	}

	abstract unfurlWithCachePolicy(
		url: string,
		nsfwMode?: MediaProxyNsfwMode,
		options?: UnfurlOptions,
	): Promise<UnfurlResult>;
}
