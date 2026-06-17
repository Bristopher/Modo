// SPDX-License-Identifier: AGPL-3.0-or-later

import {UserPremiumTypes} from '@fluxer/constants/src/UserConstants';
import type {ChannelID, MessageID} from '../BrandedTypes';
import type {ChannelService} from '../channel/services/ChannelService';
import {SYSTEM_USER_ID} from '../constants/Core';
import type {PneumaticPostDeliveryRow} from '../database/types/PneumaticPostTypes';
import type {UserCacheService} from '../infrastructure/UserCacheService';
import {Logger} from '../Logger';
import {createRequestCache, type RequestCache} from '../middleware/RequestCacheMiddleware';
import type {User} from '../models/User';
import type {UserSettings} from '../models/UserSettings';
import type {IUserRepository} from '../user/IUserRepository';
import type {UserChannelService} from '../user/services/UserChannelService';
import {
	PLUTONIUM_MOBILE_BETA_CORRECTION_DISPATCH,
	PLUTONIUM_MOBILE_BETA_DISPATCH,
	PNEUMATIC_POST_SYSTEM_NAME,
	resolvePlutoniumMobileBetaCorrectionBody,
	resolvePlutoniumMobileBetaDispatchBody,
} from './PneumaticPostNotices';
import type {PneumaticPostDeliveryRecord, PneumaticPostRepository} from './PneumaticPostRepository';

const DELIVERY_ERROR_MESSAGE_MAX_LENGTH = 1000;

interface PneumaticPostServiceDeps {
	repository: PneumaticPostRepository;
	userRepository: Pick<IUserRepository, 'findUnique'>;
	userChannelService: Pick<UserChannelService, 'ensureDmOpenForBothUsers'>;
	channelService: Pick<ChannelService, 'messages'>;
	userCacheService: UserCacheService;
}

interface SentDelivery {
	channelId: ChannelID;
	messageId: MessageID;
}

function deliveryErrorMessage(error: unknown): string {
	const message = error instanceof Error ? error.message : String(error);
	return message.slice(0, DELIVERY_ERROR_MESSAGE_MAX_LENGTH);
}

export class PneumaticPostService {
	constructor(private readonly deps: PneumaticPostServiceDeps) {}

	async considerPlutoniumMobileBetaDispatch(user: User, settings: UserSettings | null): Promise<void> {
		if (!this.isCurrentPlutoniumUser(user)) {
			return;
		}
		const locale = settings?.locale ?? user.locale ?? null;
		const originalDelivery = await this.findDelivery(user, PLUTONIUM_MOBILE_BETA_DISPATCH.key);
		if (originalDelivery === undefined) {
			return;
		}

		if (originalDelivery?.status === 'sent') {
			await this.considerPlutoniumMobileBetaCorrection(user, locale);
			return;
		}

		if (originalDelivery?.status === 'claimed') {
			return;
		}

		await this.sendRevisedPlutoniumMobileBetaDispatch(user, locale, originalDelivery);
	}

	private async findDelivery(user: User, dispatchKey: string): Promise<PneumaticPostDeliveryRow | null | undefined> {
		try {
			return await this.deps.repository.findDelivery(user.id, dispatchKey);
		} catch (error) {
			Logger.warn({dispatchKey, userId: user.id.toString(), error}, 'Failed to read Pneumatic Post delivery state');
			return undefined;
		}
	}

	private makeDeliveryRecord(user: User, dispatchKey: string, locale: string | null): PneumaticPostDeliveryRecord {
		return {
			dispatchKey,
			userId: user.id,
			claimedAt: new Date(),
			locale: locale ?? 'en-US',
		};
	}

	private async considerPlutoniumMobileBetaCorrection(user: User, locale: string | null): Promise<void> {
		const dispatchKey = PLUTONIUM_MOBILE_BETA_CORRECTION_DISPATCH.key;
		const correctionDelivery = await this.findDelivery(user, dispatchKey);
		if (
			correctionDelivery === undefined ||
			correctionDelivery?.status === 'sent' ||
			correctionDelivery?.status === 'claimed'
		) {
			return;
		}
		const deliveryRecord = this.makeDeliveryRecord(user, dispatchKey, locale);
		if (correctionDelivery === null && !(await this.tryCreateDeliveryClaim(deliveryRecord))) {
			return;
		}
		const content = resolvePlutoniumMobileBetaCorrectionBody(locale, user.id.toString());
		await this.sendClaimedDelivery({recipient: user, deliveryRecord, content});
	}

	private async sendRevisedPlutoniumMobileBetaDispatch(
		user: User,
		locale: string | null,
		existingDelivery: PneumaticPostDeliveryRow | null,
	): Promise<void> {
		const deliveryRecord = this.makeDeliveryRecord(user, PLUTONIUM_MOBILE_BETA_DISPATCH.key, locale);
		if (existingDelivery === null && !(await this.tryCreateDeliveryClaim(deliveryRecord))) {
			return;
		}
		const content = resolvePlutoniumMobileBetaDispatchBody(locale, user.id.toString());
		const sent = await this.sendClaimedDelivery({recipient: user, deliveryRecord, content});
		if (sent) {
			await this.markCorrectionImplicitlySent(user, locale, sent);
		}
	}

	private async markCorrectionImplicitlySent(user: User, locale: string | null, sent: SentDelivery): Promise<void> {
		const deliveryRecord = this.makeDeliveryRecord(user, PLUTONIUM_MOBILE_BETA_CORRECTION_DISPATCH.key, locale);
		if (!(await this.tryCreateDeliveryClaim(deliveryRecord))) {
			return;
		}
		await this.markDeliverySent(deliveryRecord, sent);
	}

	private async sendClaimedDelivery({
		recipient,
		deliveryRecord,
		content,
	}: {
		recipient: User;
		deliveryRecord: PneumaticPostDeliveryRecord;
		content: string;
	}): Promise<SentDelivery | null> {
		const requestCache = createRequestCache();
		try {
			const sent = await this.sendDelivery({recipient, content, requestCache});
			await this.markDeliverySent(deliveryRecord, sent);
			Logger.info(
				{
					system: PNEUMATIC_POST_SYSTEM_NAME,
					dispatchKey: deliveryRecord.dispatchKey,
					userId: recipient.id.toString(),
					locale: deliveryRecord.locale,
					messageId: sent.messageId.toString(),
				},
				'Pneumatic Post dispatch sent',
			);
			return sent;
		} catch (error) {
			await this.markDeliveryFailed(deliveryRecord, error);
			Logger.warn(
				{dispatchKey: deliveryRecord.dispatchKey, userId: recipient.id.toString(), error},
				'Failed to send Pneumatic Post dispatch',
			);
			return null;
		} finally {
			requestCache.clear();
		}
	}

	private async tryCreateDeliveryClaim(record: PneumaticPostDeliveryRecord): Promise<boolean> {
		try {
			return await this.deps.repository.tryCreateDeliveryClaim(record);
		} catch (error) {
			Logger.warn(
				{dispatchKey: record.dispatchKey, userId: record.userId.toString(), error},
				'Failed to claim Pneumatic Post delivery',
			);
			return false;
		}
	}

	private async sendDelivery({
		recipient,
		content,
		requestCache,
	}: {
		recipient: User;
		content: string;
		requestCache: RequestCache;
	}): Promise<SentDelivery> {
		const systemUser = await this.deps.userRepository.findUnique(SYSTEM_USER_ID);
		if (!systemUser) {
			throw new Error('System user (id=0) not found');
		}
		const channel = await this.deps.userChannelService.ensureDmOpenForBothUsers({
			userId: systemUser.id,
			recipientId: recipient.id,
			userCacheService: this.deps.userCacheService,
			requestCache,
		});
		const message = await this.deps.channelService.messages.send.sendMessage({
			user: systemUser,
			channelId: channel.id,
			data: {content},
			requestCache,
		});
		return {channelId: channel.id, messageId: message.id};
	}

	private async markDeliverySent(record: PneumaticPostDeliveryRecord, sent: SentDelivery): Promise<void> {
		try {
			await this.deps.repository.markDeliverySent({...record, ...sent, sentAt: new Date()});
		} catch (error) {
			Logger.warn(
				{
					system: PNEUMATIC_POST_SYSTEM_NAME,
					dispatchKey: record.dispatchKey,
					userId: record.userId.toString(),
					messageId: sent.messageId.toString(),
					error,
				},
				'Pneumatic Post dispatch sent but delivery record was not finalized',
			);
		}
	}

	private async markDeliveryFailed(record: PneumaticPostDeliveryRecord, error: unknown): Promise<void> {
		await this.deps.repository
			.markDeliveryFailed({...record, errorMessage: deliveryErrorMessage(error)})
			.catch((recordError) => {
				Logger.warn(
					{dispatchKey: record.dispatchKey, userId: record.userId.toString(), recordError},
					'Failed to record Pneumatic Post delivery failure',
				);
			});
	}

	private isCurrentPlutoniumUser(user: User): boolean {
		return user.premiumType !== null && user.premiumType !== UserPremiumTypes.LIFETIME && user.isPremium();
	}
}
