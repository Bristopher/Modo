// SPDX-License-Identifier: AGPL-3.0-or-later

import {MAX_BIO_LENGTH} from '@fluxer/constants/src/LimitConstants';
import {PremiumFlags, ProfileFieldPrivacyFlags, UserFlags} from '@fluxer/constants/src/UserConstants';
import {ValidationErrorCodes} from '@fluxer/constants/src/ValidationErrorCodes';
import {isSupportedTimeZoneId} from '@fluxer/date_utils/src/TimeZoneUtils';
import {ContentBlockedError} from '@fluxer/errors/src/domains/content/ContentBlockedError';
import {InputValidationError} from '@fluxer/errors/src/domains/core/InputValidationError';
import {MissingAccessError} from '@fluxer/errors/src/domains/core/MissingAccessError';
import type {UserUpdateRequest} from '@fluxer/schema/src/domains/user/UserRequestSchemas';
import type {IRateLimitService} from '@pkgs/rate_limit/src/IRateLimitService';
import {ms} from 'itty-time';
import type {UserRow} from '../../database/types/UserTypes';
import type {IGuildRepositoryAggregate} from '../../guild/repositories/IGuildRepositoryAggregate';
import {contentModerationService} from '../../infrastructure/ContentModerationService';
import type {EntityAssetService, PreparedAssetUpload} from '../../infrastructure/EntityAssetService';
import type {LimitConfigService} from '../../limits/LimitConfigService';
import {resolveLimitSafe} from '../../limits/LimitConfigUtils';
import {createLimitMatchContext} from '../../limits/LimitMatchContextBuilder';
import {profileSubstringBlocklistCache} from '../../middleware/ProfileSubstringBlocklistCache';
import type {User} from '../../models/User';
import {deriveDominantAvatarColor} from '../../utils/AvatarColorUtils';
import * as EmojiUtils from '../../utils/EmojiUtils';
import type {IUserAccountRepository} from '../repositories/IUserAccountRepository';
import {canUseProfileTimezone, isProfileSubstringExempt} from '../UserHelpers';
import type {UserAccountUpdatePropagator} from './UserAccountUpdatePropagator';

interface UserUpdateMetadata {
	invalidateAuthSessions?: boolean;
}

type UserFieldUpdates = Partial<UserRow>;

interface ProfileUpdateResult {
	updates: UserFieldUpdates;
	metadata: UserUpdateMetadata;
	preparedAvatarUpload: PreparedAssetUpload | null;
	preparedBannerUpload: PreparedAssetUpload | null;
}

interface UserAccountProfileServiceDeps {
	userAccountRepository: IUserAccountRepository;
	guildRepository: IGuildRepositoryAggregate;
	entityAssetService: EntityAssetService;
	rateLimitService: IRateLimitService;
	updatePropagator: UserAccountUpdatePropagator;
	limitConfigService: LimitConfigService;
}

export class UserAccountProfileService {
	constructor(private readonly deps: UserAccountProfileServiceDeps) {}

	async processProfileUpdates(params: {user: User; data: UserUpdateRequest}): Promise<ProfileUpdateResult> {
		const {user, data} = params;
		const updates: UserFieldUpdates = {
			avatar_hash: user.avatarHash,
			banner_hash: user.bannerHash,
			flags: user.flags,
		};
		const metadata: UserUpdateMetadata = {};
		let preparedAvatarUpload: PreparedAssetUpload | null = null;
		let preparedBannerUpload: PreparedAssetUpload | null = null;
		if (data.bio !== undefined) {
			contentModerationService.scanText(data.bio, {
				userId: user.id,
				guildId: null,
				channelId: null,
				messageId: null,
				surface: 'profile_field',
			});
			await this.processBioUpdate({user, bio: data.bio, updates});
		}
		if (data.pronouns !== undefined) {
			contentModerationService.scanText(data.pronouns, {
				userId: user.id,
				guildId: null,
				channelId: null,
				messageId: null,
				surface: 'profile_field',
			});
			await this.processPronounsUpdate({user, pronouns: data.pronouns, updates});
		}
		if (data.accent_color !== undefined) {
			await this.processAccentColorUpdate({user, accentColor: data.accent_color, updates});
		}
		const canUpdateProfileTimezone = canUseProfileTimezone(user);
		if (canUpdateProfileTimezone && data.timezone !== undefined) {
			const nextTimezone = this.processTimezoneUpdate({user, timezone: data.timezone, updates});
			if (nextTimezone !== null && user.timezone === null && data.timezone_privacy_flags === undefined) {
				updates.timezone_privacy_flags = ProfileFieldPrivacyFlags.EVERYONE;
			}
		}
		if (canUpdateProfileTimezone && data.timezone_privacy_flags !== undefined) {
			this.processTimezonePrivacyFlagsUpdate({
				user,
				privacyFlags: data.timezone_privacy_flags,
				updates,
			});
		}
		if (data.avatar !== undefined) {
			preparedAvatarUpload = await this.processAvatarUpdate({user, avatar: data.avatar, updates});
		}
		if (data.banner !== undefined) {
			try {
				preparedBannerUpload = await this.processBannerUpdate({user, banner: data.banner, updates});
			} catch (error) {
				if (preparedAvatarUpload) {
					await this.deps.entityAssetService.rollbackAssetUpload(preparedAvatarUpload);
				}
				throw error;
			}
		}
		if (!user.isBot) {
			this.processPremiumBadgeFlags({user, data, updates});
			this.processPremiumOnboardingDismissal({user, data, updates});
			this.processGiftInventoryRead({user, data, updates});
		}
		if (data.mention_flags !== undefined) {
			updates.mention_flags = data.mention_flags;
		}
		return {updates, metadata, preparedAvatarUpload, preparedBannerUpload};
	}

	async commitAssetChanges(result: ProfileUpdateResult): Promise<void> {
		if (result.preparedAvatarUpload) {
			await this.deps.entityAssetService.commitAssetChange({
				prepared: result.preparedAvatarUpload,
				deferDeletion: true,
			});
		}
		if (result.preparedBannerUpload) {
			await this.deps.entityAssetService.commitAssetChange({
				prepared: result.preparedBannerUpload,
				deferDeletion: true,
			});
		}
	}

	async rollbackAssetChanges(result: ProfileUpdateResult): Promise<void> {
		if (result.preparedAvatarUpload) {
			await this.deps.entityAssetService.rollbackAssetUpload(result.preparedAvatarUpload);
		}
		if (result.preparedBannerUpload) {
			await this.deps.entityAssetService.rollbackAssetUpload(result.preparedBannerUpload);
		}
	}

	private async processBioUpdate(params: {user: User; bio: string | null; updates: UserFieldUpdates}): Promise<void> {
		const {user, bio, updates} = params;
		if (bio !== user.bio) {
			const bioRateLimit = await this.deps.rateLimitService.checkLimit({
				identifier: `bio_change:${user.id}`,
				maxAttempts: 25,
				windowMs: ms('30 minutes'),
			});
			if (!bioRateLimit.allowed) {
				const minutes = Math.ceil((bioRateLimit.retryAfter || 0) / 60);
				throw InputValidationError.fromCode('bio', ValidationErrorCodes.BIO_CHANGED_TOO_MANY_TIMES, {minutes});
			}
			const ctx = createLimitMatchContext({user});
			const maxBioLength = resolveLimitSafe(
				this.deps.limitConfigService.getConfigSnapshot(),
				ctx,
				'max_bio_length',
				MAX_BIO_LENGTH,
			);
			if (bio && bio.length > maxBioLength) {
				throw InputValidationError.fromCode('bio', ValidationErrorCodes.CONTENT_EXCEEDS_MAX_LENGTH, {
					maxLength: maxBioLength,
				});
			}
			let sanitizedBio = bio;
			if (bio) {
				sanitizedBio = await EmojiUtils.sanitizeCustomEmojis({
					content: bio,
					userId: user.id,
					webhookId: null,
					guildId: null,
					userRepository: this.deps.userAccountRepository,
					guildRepository: this.deps.guildRepository,
					limitConfigService: this.deps.limitConfigService,
				});
			}
			if (
				sanitizedBio &&
				!isProfileSubstringExempt(user) &&
				profileSubstringBlocklistCache.containsBannedSubstring('bio', sanitizedBio)
			) {
				throw new ContentBlockedError();
			}
			updates.bio = sanitizedBio;
		}
	}

	private async processPronounsUpdate(params: {
		user: User;
		pronouns: string | null;
		updates: UserFieldUpdates;
	}): Promise<void> {
		const {user, pronouns, updates} = params;
		if (pronouns !== user.pronouns) {
			const pronounsRateLimit = await this.deps.rateLimitService.checkLimit({
				identifier: `pronouns_change:${user.id}`,
				maxAttempts: 25,
				windowMs: ms('30 minutes'),
			});
			if (!pronounsRateLimit.allowed) {
				const minutes = Math.ceil((pronounsRateLimit.retryAfter || 0) / 60);
				throw InputValidationError.fromCode('pronouns', ValidationErrorCodes.PRONOUNS_CHANGED_TOO_MANY_TIMES, {
					minutes,
				});
			}
			if (
				pronouns &&
				!isProfileSubstringExempt(user) &&
				profileSubstringBlocklistCache.containsBannedSubstring('pronouns', pronouns)
			) {
				throw new ContentBlockedError();
			}
			updates.pronouns = pronouns;
		}
	}

	private async processAccentColorUpdate(params: {
		user: User;
		accentColor: number | null;
		updates: UserFieldUpdates;
	}): Promise<void> {
		const {user, accentColor, updates} = params;
		if (accentColor !== user.accentColor) {
			const accentColorRateLimit = await this.deps.rateLimitService.checkLimit({
				identifier: `accent_color_change:${user.id}`,
				maxAttempts: 25,
				windowMs: ms('30 minutes'),
			});
			if (!accentColorRateLimit.allowed) {
				const minutes = Math.ceil((accentColorRateLimit.retryAfter || 0) / 60);
				throw InputValidationError.fromCode('accent_color', ValidationErrorCodes.ACCENT_COLOR_CHANGED_TOO_MANY_TIMES, {
					minutes,
				});
			}
			updates.accent_color = accentColor;
		}
	}

	private processTimezoneUpdate(params: {
		user: User;
		timezone: string | null;
		updates: UserFieldUpdates;
	}): string | null {
		const {user, timezone, updates} = params;
		const nextTimezone = timezone?.trim() || null;
		if (nextTimezone !== null && !isSupportedTimeZoneId(nextTimezone)) {
			throw InputValidationError.fromCode('timezone', ValidationErrorCodes.INVALID_TIMEZONE_IDENTIFIER);
		}
		if (nextTimezone !== user.timezone) {
			updates.timezone = nextTimezone;
		}
		return nextTimezone;
	}

	private processTimezonePrivacyFlagsUpdate(params: {
		user: User;
		privacyFlags: number;
		updates: UserFieldUpdates;
	}): void {
		const {user, privacyFlags, updates} = params;
		if (privacyFlags !== user.timezonePrivacyFlags) {
			updates.timezone_privacy_flags = privacyFlags;
		}
	}

	private async processAvatarUpdate(params: {
		user: User;
		avatar: string | null;
		updates: UserFieldUpdates;
	}): Promise<PreparedAssetUpload | null> {
		const {user, avatar, updates} = params;
		if (avatar === null) {
			updates.avatar_hash = null;
			updates.avatar_color = null;
			if (user.avatarHash) {
				return await this.deps.entityAssetService.prepareAssetUpload({
					assetType: 'avatar',
					entityType: 'user',
					entityId: user.id,
					previousHash: user.avatarHash,
					base64Image: null,
					errorPath: 'avatar',
				});
			}
			return null;
		}
		const avatarRateLimit = await this.deps.rateLimitService.checkLimit({
			identifier: `avatar_change:${user.id}`,
			maxAttempts: 25,
			windowMs: ms('30 minutes'),
		});
		if (!avatarRateLimit.allowed) {
			const minutes = Math.ceil((avatarRateLimit.retryAfter || 0) / 60);
			throw InputValidationError.fromCode('avatar', ValidationErrorCodes.AVATAR_CHANGED_TOO_MANY_TIMES, {minutes});
		}
		const prepared = await this.deps.entityAssetService.prepareAssetUpload({
			assetType: 'avatar',
			entityType: 'user',
			entityId: user.id,
			previousHash: user.avatarHash,
			base64Image: avatar,
			errorPath: 'avatar',
		});
		const ctx = createLimitMatchContext({user});
		const hasAnimatedAvatar = resolveLimitSafe(
			this.deps.limitConfigService.getConfigSnapshot(),
			ctx,
			'feature_animated_avatar',
			0,
		);
		if (prepared.isAnimated && hasAnimatedAvatar === 0) {
			await this.deps.entityAssetService.rollbackAssetUpload(prepared);
			throw InputValidationError.fromCode('avatar', ValidationErrorCodes.ANIMATED_AVATARS_REQUIRE_PREMIUM);
		}
		if (prepared.imageBuffer) {
			const derivedColor = await deriveDominantAvatarColor(prepared.imageBuffer);
			if (derivedColor !== user.avatarColor) {
				updates.avatar_color = derivedColor;
			}
		}
		if (prepared.newHash !== user.avatarHash) {
			updates.avatar_hash = prepared.newHash;
			return prepared;
		}
		return null;
	}

	private async processBannerUpdate(params: {
		user: User;
		banner: string | null;
		updates: UserFieldUpdates;
	}): Promise<PreparedAssetUpload | null> {
		const {user, banner, updates} = params;
		if (banner === null) {
			updates.banner_color = null;
		}
		const ctx = createLimitMatchContext({user});
		const hasAnimatedBanner = resolveLimitSafe(
			this.deps.limitConfigService.getConfigSnapshot(),
			ctx,
			'feature_animated_banner',
			0,
		);
		if (banner !== null && hasAnimatedBanner === 0) {
			throw InputValidationError.fromCode('banner', ValidationErrorCodes.BANNERS_REQUIRE_PREMIUM);
		}
		const bannerRateLimit = await this.deps.rateLimitService.checkLimit({
			identifier: `banner_change:${user.id}`,
			maxAttempts: 25,
			windowMs: ms('30 minutes'),
		});
		if (!bannerRateLimit.allowed) {
			const minutes = Math.ceil((bannerRateLimit.retryAfter || 0) / 60);
			throw InputValidationError.fromCode('banner', ValidationErrorCodes.BANNER_CHANGED_TOO_MANY_TIMES, {minutes});
		}
		const prepared = await this.deps.entityAssetService.prepareAssetUpload({
			assetType: 'banner',
			entityType: 'user',
			entityId: user.id,
			previousHash: user.bannerHash,
			base64Image: banner,
			errorPath: 'banner',
		});
		if (prepared.isAnimated && hasAnimatedBanner === 0) {
			await this.deps.entityAssetService.rollbackAssetUpload(prepared);
			throw InputValidationError.fromCode('banner', ValidationErrorCodes.ANIMATED_AVATARS_REQUIRE_PREMIUM);
		}
		if (banner !== null && prepared.imageBuffer) {
			const derivedColor = await deriveDominantAvatarColor(prepared.imageBuffer);
			if (derivedColor !== user.bannerColor) {
				updates.banner_color = derivedColor;
			}
		}
		if (prepared.newHash !== user.bannerHash) {
			updates.banner_hash = prepared.newHash;
			return prepared;
		}
		return null;
	}

	private processPremiumBadgeFlags(params: {user: User; data: UserUpdateRequest; updates: UserFieldUpdates}): void {
		const {user, data, updates} = params;
		let flagsUpdated = false;
		let newPremiumFlags = user.premiumFlags;
		if (data.premium_badge_hidden !== undefined) {
			if (data.premium_badge_hidden) {
				newPremiumFlags = newPremiumFlags | PremiumFlags.BADGE_HIDDEN;
			} else {
				newPremiumFlags = newPremiumFlags & ~PremiumFlags.BADGE_HIDDEN;
			}
			flagsUpdated = true;
		}
		if (data.premium_badge_masked !== undefined) {
			if (data.premium_badge_masked) {
				newPremiumFlags = newPremiumFlags | PremiumFlags.BADGE_MASKED;
			} else {
				newPremiumFlags = newPremiumFlags & ~PremiumFlags.BADGE_MASKED;
			}
			flagsUpdated = true;
		}
		if (data.premium_badge_timestamp_hidden !== undefined) {
			if (data.premium_badge_timestamp_hidden) {
				newPremiumFlags = newPremiumFlags | PremiumFlags.BADGE_TIMESTAMP_HIDDEN;
			} else {
				newPremiumFlags = newPremiumFlags & ~PremiumFlags.BADGE_TIMESTAMP_HIDDEN;
			}
			flagsUpdated = true;
		}
		if (data.premium_badge_sequence_hidden !== undefined) {
			if (data.premium_badge_sequence_hidden) {
				newPremiumFlags = newPremiumFlags | PremiumFlags.BADGE_SEQUENCE_HIDDEN;
			} else {
				newPremiumFlags = newPremiumFlags & ~PremiumFlags.BADGE_SEQUENCE_HIDDEN;
			}
			flagsUpdated = true;
		}
		if (data.premium_enabled_override !== undefined) {
			if (!(user.flags & UserFlags.STAFF)) {
				throw new MissingAccessError();
			}
			if (data.premium_enabled_override) {
				newPremiumFlags = newPremiumFlags | PremiumFlags.ENABLED_OVERRIDE;
			} else {
				newPremiumFlags = newPremiumFlags & ~PremiumFlags.ENABLED_OVERRIDE;
			}
			flagsUpdated = true;
		}
		if (flagsUpdated) {
			updates.premium_flags = newPremiumFlags;
		}
	}

	private processPremiumOnboardingDismissal(params: {
		user: User;
		data: UserUpdateRequest;
		updates: UserFieldUpdates;
	}): void {
		const {data, updates} = params;
		if (data.has_dismissed_premium_onboarding !== undefined) {
			if (data.has_dismissed_premium_onboarding) {
				updates.premium_onboarding_dismissed_at = new Date();
			}
		}
	}

	private processGiftInventoryRead(params: {user: User; data: UserUpdateRequest; updates: UserFieldUpdates}): void {
		const {user, data, updates} = params;
		if (data.has_unread_gift_inventory === false) {
			updates.gift_inventory_client_seq = user.giftInventoryServerSeq;
		}
	}
}
