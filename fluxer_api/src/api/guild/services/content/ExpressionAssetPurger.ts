// SPDX-License-Identifier: AGPL-3.0-or-later

import {getExtensionWhitelist} from '@fluxer/constants/src/AssetFormatPolicy';
import {Config} from '../../../Config';
import type {IAssetDeletionQueue} from '../../../infrastructure/IAssetDeletionQueue';

const STICKER_EXTENSIONS = getExtensionWhitelist('sticker');

export class ExpressionAssetPurger {
	constructor(private readonly assetDeletionQueue: IAssetDeletionQueue) {}

	async purgeEmoji(id: string): Promise<void> {
		await this.queueAsset('emojis', id, this.buildEmojiCdnUrls(id));
	}

	async purgeSticker(id: string): Promise<void> {
		await this.queueAsset('stickers', id, this.buildStickerCdnUrls(id));
	}

	private async queueAsset(prefix: string, id: string, cdnUrls: Array<string>): Promise<void> {
		const uniqueUrls = Array.from(new Set(cdnUrls));
		const [primaryUrl, ...additionalUrls] = uniqueUrls;
		await this.assetDeletionQueue.queueDeletion({
			s3Key: `${prefix}/${id}`,
			cdnUrl: primaryUrl ?? null,
			reason: 'asset_purge',
		});
		await Promise.all(additionalUrls.map((url) => this.assetDeletionQueue.queueCdnPurge(url)));
	}

	private buildEmojiCdnUrls(id: string): Array<string> {
		const base = Config.endpoints.media;
		return [`${base}/emojis/${id}.webp`, `${base}/emojis/${id}.gif`];
	}

	private buildStickerCdnUrls(id: string): Array<string> {
		const base = Config.endpoints.media;
		return STICKER_EXTENSIONS.map((ext) => `${base}/stickers/${id}.${ext}`);
	}
}
