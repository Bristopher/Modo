// SPDX-License-Identifier: AGPL-3.0-or-later

import {DeletionReasons} from '@fluxer/constants/src/Core';
import {UserFlags} from '@fluxer/constants/src/UserConstants';
import {UserOwnsGuildsError} from '@fluxer/errors/src/domains/guild/UserOwnsGuildsError';
import {UnknownUserError} from '@fluxer/errors/src/domains/user/UnknownUserError';
import type {IEmailService} from '@pkgs/email/src/IEmailService';
import {ms} from 'itty-time';
import type {ApiContext} from '../../ApiContext';
import * as AuthSession from '../../auth/AuthSession';
import type {UserID} from '../../BrandedTypes';
import {Config} from '../../Config';
import type {IGuildRepositoryAggregate} from '../../guild/repositories/IGuildRepositoryAggregate';
import type {KVAccountDeletionQueueService} from '../../infrastructure/KVAccountDeletionQueueService';
import type {IUserAccountRepository} from '../repositories/IUserAccountRepository';
import {hasPartialUserFieldsChanged} from '../UserMappers';
import {reschedulePendingDeletion} from './PendingDeletionCoordinator';
import type {UserAccountUpdatePropagator} from './UserAccountUpdatePropagator';

interface UserAccountLifecycleServiceDeps {
	apiContext: ApiContext;
	userAccountRepository: IUserAccountRepository;
	guildRepository: IGuildRepositoryAggregate;
	emailService: IEmailService;
	updatePropagator: UserAccountUpdatePropagator;
	kvDeletionQueue: KVAccountDeletionQueueService;
}

export class UserAccountLifecycleService {
	constructor(private readonly deps: UserAccountLifecycleServiceDeps) {}

	async selfDisable(userId: UserID): Promise<void> {
		const user = await this.deps.userAccountRepository.findUnique(userId);
		if (!user) {
			throw new UnknownUserError();
		}
		const updatedUser = await this.deps.userAccountRepository.patchUpsert(
			userId,
			{
				flags: user.flags | UserFlags.DISABLED,
			},
			user.toRow(),
		);
		await AuthSession.terminateAllUserSessions(this.deps.apiContext, userId);
		if (updatedUser) {
			await this.deps.updatePropagator.dispatchUserUpdate(updatedUser);
			if (hasPartialUserFieldsChanged(user, updatedUser)) {
				await this.deps.updatePropagator.updateUserCache(updatedUser);
			}
		}
	}

	async selfDelete(userId: UserID): Promise<void> {
		const user = await this.deps.userAccountRepository.findUnique(userId);
		if (!user) {
			throw new UnknownUserError();
		}
		const ownedGuildIds = await this.deps.guildRepository.listOwnedGuildIds(userId);
		if (ownedGuildIds.length > 0) {
			throw new UserOwnsGuildsError();
		}
		const gracePeriodMs = Config.deletionGracePeriodHours * ms('1 hour');
		const pendingDeletionAt = new Date(Date.now() + gracePeriodMs);
		const updatedUser = await this.deps.userAccountRepository.patchUpsert(
			userId,
			{
				flags: user.flags | UserFlags.SELF_DELETED,
				pending_deletion_at: pendingDeletionAt,
				deletion_reason_code: DeletionReasons.USER_REQUESTED,
			},
			user.toRow(),
		);
		await reschedulePendingDeletion({
			userId,
			currentPendingDeletionAt: user.pendingDeletionAt,
			nextPendingDeletionAt: pendingDeletionAt,
			deletionReasonCode: DeletionReasons.USER_REQUESTED,
			userRepository: this.deps.userAccountRepository,
			deletionQueue: this.deps.kvDeletionQueue,
		});
		if (user.email) {
			await this.deps.emailService.sendSelfDeletionScheduledEmail(
				user.email,
				user.username,
				pendingDeletionAt,
				user.locale,
			);
		}
		await AuthSession.terminateAllUserSessions(this.deps.apiContext, userId);
		if (updatedUser) {
			await this.deps.updatePropagator.dispatchUserUpdate(updatedUser);
			if (hasPartialUserFieldsChanged(user, updatedUser)) {
				await this.deps.updatePropagator.updateUserCache(updatedUser);
			}
		}
	}
}
