// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';
import type {IGatewayService} from '../infrastructure/IGatewayService';
import type {Application} from '../models/Application';
import type {User} from '../models/User';
import type {IUserRepository} from '../user/IUserRepository';
import {mapUserToPrivateResponse} from '../user/UserMappers';
import type {IApplicationRepository} from './repositories/IApplicationRepository';

export class BotMfaMirrorService {
	constructor(
		private readonly applicationRepository: IApplicationRepository,
		private readonly userRepository: IUserRepository,
		private readonly gatewayService: IGatewayService,
	) {}

	private cloneAuthenticatorTypes(source: User): Set<number> {
		return source.authenticatorTypes ? new Set(source.authenticatorTypes) : new Set();
	}

	private hasSameAuthenticatorTypes(target: User, desired: Set<number>): boolean {
		const current = target.authenticatorTypes ?? new Set<number>();
		if (current.size !== desired.size) return false;
		for (const value of current) {
			if (!desired.has(value)) {
				return false;
			}
		}
		return true;
	}

	private async listApplications(ownerUserId: UserID): Promise<Array<Application>> {
		return this.applicationRepository.listApplicationsByOwner(ownerUserId);
	}

	async syncAuthenticatorTypesForOwner(owner: User): Promise<void> {
		if (owner.isBot) return;
		const desiredTypes = this.cloneAuthenticatorTypes(owner);
		const applications = await this.listApplications(owner.id);
		await Promise.all(
			applications.map(async (application) => {
				if (!application.hasBotUser()) return;
				const botUserId = application.getBotUserId();
				if (!botUserId) return;
				const botUser = await this.userRepository.findUnique(botUserId);
				if (!botUser) return;
				if (this.hasSameAuthenticatorTypes(botUser, desiredTypes)) {
					return;
				}
				const updatedBotUser = await this.userRepository.patchUpsert(
					botUserId,
					{
						authenticator_types: desiredTypes,
					},
					botUser.toRow(),
				);
				if (updatedBotUser) {
					await this.gatewayService.dispatchPresence({
						userId: botUserId,
						event: 'USER_UPDATE',
						data: mapUserToPrivateResponse(updatedBotUser),
					});
				}
			}),
		);
	}
}
