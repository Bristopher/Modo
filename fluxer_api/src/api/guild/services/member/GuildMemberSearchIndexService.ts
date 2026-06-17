// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../../../BrandedTypes';
import {Logger} from '../../../Logger';
import type {GuildMember} from '../../../models/GuildMember';
import type {User} from '../../../models/User';
import {getGuildMemberSearchService} from '../../../SearchFactory';
import type {IGuildMemberSearchService} from '../../../search/IGuildMemberSearchService';

interface GuildMemberSearchIndexOptions {
	includeDefault?: boolean;
}

function getGuildMemberIndexServices(options: GuildMemberSearchIndexOptions = {}): Array<IGuildMemberSearchService> {
	const services: Array<IGuildMemberSearchService> = [];
	const includeDefault = options.includeDefault ?? true;
	const defaultService = getGuildMemberSearchService();
	if (includeDefault && defaultService) {
		services.push(defaultService);
	}
	return services;
}

export class GuildMemberSearchIndexService {
	async indexMember(member: GuildMember, user: User, options?: GuildMemberSearchIndexOptions): Promise<void> {
		try {
			const searchServices = getGuildMemberIndexServices(options);
			if (searchServices.length === 0) {
				return;
			}
			await Promise.all(searchServices.map((searchService) => searchService.indexMember(member, user)));
		} catch (error) {
			Logger.error(
				{
					guildId: member.guildId.toString(),
					userId: member.userId.toString(),
					error,
				},
				'Failed to index guild member in search',
			);
		}
	}

	async updateMember(member: GuildMember, user: User, options?: GuildMemberSearchIndexOptions): Promise<void> {
		try {
			const searchServices = getGuildMemberIndexServices(options);
			if (searchServices.length === 0) {
				return;
			}
			await Promise.all(searchServices.map((searchService) => searchService.updateMember(member, user)));
		} catch (error) {
			Logger.error(
				{
					guildId: member.guildId.toString(),
					userId: member.userId.toString(),
					error,
				},
				'Failed to update guild member in search index',
			);
		}
	}

	async deleteMember(guildId: GuildID, userId: UserID, options?: GuildMemberSearchIndexOptions): Promise<void> {
		try {
			const searchServices = getGuildMemberIndexServices(options);
			if (searchServices.length === 0) {
				return;
			}
			await Promise.all(searchServices.map((searchService) => searchService.deleteMember(guildId, userId)));
		} catch (error) {
			Logger.error(
				{
					guildId: guildId.toString(),
					userId: userId.toString(),
					error,
				},
				'Failed to delete guild member from search index',
			);
		}
	}
}
