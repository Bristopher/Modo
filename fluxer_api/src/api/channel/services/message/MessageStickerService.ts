// SPDX-License-Identifier: AGPL-3.0-or-later

import {Permissions} from '@fluxer/constants/src/ChannelConstants';
import {ValidationErrorCodes} from '@fluxer/constants/src/ValidationErrorCodes';
import {InputValidationError} from '@fluxer/errors/src/domains/core/InputValidationError';
import {MissingPermissionsError} from '@fluxer/errors/src/domains/core/MissingPermissionsError';
import {NsfwEmojiStickerBlockedError} from '@fluxer/errors/src/domains/moderation/NsfwEmojiStickerBlockedError';
import type {GuildID, StickerID, UserID} from '../../../BrandedTypes';
import type {MessageStickerItem} from '../../../database/types/MessageTypes';
import type {IGuildRepositoryAggregate} from '../../../guild/repositories/IGuildRepositoryAggregate';
import type {LimitConfigService} from '../../../limits/LimitConfigService';
import {resolveLimitSafe} from '../../../limits/LimitConfigUtils';
import {createLimitMatchContext} from '../../../limits/LimitMatchContextBuilder';
import type {PackService} from '../../../pack/PackService';
import type {IUserRepository} from '../../../user/IUserRepository';

export class MessageStickerService {
	constructor(
		private userRepository: IUserRepository,
		private guildRepository: IGuildRepositoryAggregate,
		private packService: PackService,
		private readonly limitConfigService: LimitConfigService,
	) {}

	async computeStickerIds(params: {
		stickerIds: Array<StickerID>;
		userId: UserID | null;
		guildId: GuildID | null;
		hasPermission?: (permission: bigint) => Promise<boolean>;
		isNSFWAllowed?: boolean;
	}): Promise<Array<MessageStickerItem>> {
		const {stickerIds, userId, guildId, hasPermission, isNSFWAllowed = true} = params;
		const packResolver = await this.packService.createPackExpressionAccessResolver({
			userId,
			type: 'sticker',
		});
		let hasGlobalExpressions = 0;
		if (userId) {
			const user = await this.userRepository.findUnique(userId);
			const ctx = createLimitMatchContext({user});
			hasGlobalExpressions = resolveLimitSafe(
				this.limitConfigService.getConfigSnapshot(),
				ctx,
				'feature_global_expressions',
				0,
			);
		}
		return Promise.all(
			stickerIds.map(async (stickerId) => {
				if (!guildId) {
					if (hasGlobalExpressions === 0) {
						throw InputValidationError.fromCode('sticker', ValidationErrorCodes.CUSTOM_STICKERS_IN_DMS_REQUIRE_PREMIUM);
					}
					const stickerFromAnyGuild = await this.guildRepository.getStickerById(stickerId);
					if (!stickerFromAnyGuild) {
						throw InputValidationError.fromCode('sticker', ValidationErrorCodes.CUSTOM_STICKER_NOT_FOUND);
					}
					const packAccess = await packResolver.resolve(stickerFromAnyGuild.guildId);
					if (packAccess === 'not-accessible') {
						throw InputValidationError.fromCode('sticker', ValidationErrorCodes.CUSTOM_STICKER_NOT_FOUND);
					}
					if (!isNSFWAllowed && stickerFromAnyGuild.isNsfw) {
						throw new NsfwEmojiStickerBlockedError();
					}
					return {
						sticker_id: stickerFromAnyGuild.id,
						name: stickerFromAnyGuild.name,
						animated: stickerFromAnyGuild.animated,
						...(stickerFromAnyGuild.isNsfw ? {nsfw: true} : {}),
					};
				}
				const guildSticker = await this.guildRepository.getSticker(stickerId, guildId);
				if (guildSticker) {
					if (!isNSFWAllowed && guildSticker.isNsfw) {
						throw new NsfwEmojiStickerBlockedError();
					}
					return {
						sticker_id: guildSticker.id,
						name: guildSticker.name,
						animated: guildSticker.animated,
						...(guildSticker.isNsfw ? {nsfw: true} : {}),
					};
				}
				const stickerFromOtherGuild = await this.guildRepository.getStickerById(stickerId);
				if (!stickerFromOtherGuild) {
					throw InputValidationError.fromCode('sticker', ValidationErrorCodes.CUSTOM_STICKER_NOT_FOUND);
				}
				if (hasGlobalExpressions === 0) {
					throw InputValidationError.fromCode(
						'sticker',
						ValidationErrorCodes.CUSTOM_STICKERS_REQUIRE_PREMIUM_OUTSIDE_SOURCE,
					);
				}
				if (hasPermission) {
					const canUseExternalStickers = await hasPermission(Permissions.USE_EXTERNAL_STICKERS);
					if (!canUseExternalStickers) {
						throw new MissingPermissionsError();
					}
				}
				const packAccess = await packResolver.resolve(stickerFromOtherGuild.guildId);
				if (packAccess === 'not-accessible') {
					throw InputValidationError.fromCode('sticker', ValidationErrorCodes.CUSTOM_STICKER_NOT_FOUND);
				}
				if (!isNSFWAllowed && stickerFromOtherGuild.isNsfw) {
					throw new NsfwEmojiStickerBlockedError();
				}
				return {
					sticker_id: stickerFromOtherGuild.id,
					name: stickerFromOtherGuild.name,
					animated: stickerFromOtherGuild.animated,
					...(stickerFromOtherGuild.isNsfw ? {nsfw: true} : {}),
				};
			}),
		);
	}
}
