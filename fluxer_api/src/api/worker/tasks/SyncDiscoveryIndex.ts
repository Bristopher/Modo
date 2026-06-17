// SPDX-License-Identifier: AGPL-3.0-or-later

import {DiscoveryApplicationStatus} from '@fluxer/constants/src/DiscoveryConstants';
import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import type {GuildID} from '../../BrandedTypes';
import {GuildDiscoveryRepository} from '../../guild/repositories/GuildDiscoveryRepository';
import {getGuildSearchService} from '../../SearchFactory';
import {getWorkerDependencies} from '../WorkerContext';

const BATCH_SIZE = 50;
const syncDiscoveryIndex: WorkerTaskHandler = async (_payload, helpers) => {
	helpers.logger.info('Starting discovery index sync');
	const guildSearchService = getGuildSearchService();
	if (!guildSearchService) {
		helpers.logger.warn('Search service not available, skipping discovery index sync');
		return;
	}
	const {guildRepository, gatewayService} = getWorkerDependencies();
	const discoveryRepository = new GuildDiscoveryRepository();
	const approvedRows = await discoveryRepository.listByStatus(DiscoveryApplicationStatus.APPROVED, 1000);
	if (approvedRows.length === 0) {
		helpers.logger.info('No discoverable guilds to sync');
		return;
	}
	const guildIds = approvedRows.map((row) => row.guild_id);
	let freshCounts = new Map<
		GuildID,
		{
			memberCount: number;
			onlineCount: number;
		}
	>();
	try {
		freshCounts = await gatewayService.getDiscoveryGuildCounts(guildIds);
	} catch (error) {
		helpers.logger.warn(
			{error: error instanceof Error ? error.message : String(error)},
			'Failed to fetch fresh guild counts from gateway, using database values',
		);
	}
	let synced = 0;
	for (let i = 0; i < guildIds.length; i += BATCH_SIZE) {
		const batch = guildIds.slice(i, i + BATCH_SIZE);
		for (const guildId of batch) {
			const guild = await guildRepository.findUnique(guildId);
			if (!guild) continue;
			const discoveryRow = await discoveryRepository.findByGuildId(guildId);
			if (!discoveryRow || discoveryRow.status !== DiscoveryApplicationStatus.APPROVED) continue;
			const counts = freshCounts.get(guildId);
			await guildSearchService.updateGuild(guild, {
				description: discoveryRow.description,
				categoryId: discoveryRow.category_type,
				primaryLanguage: discoveryRow.primary_language ?? null,
				tags: discoveryRow.custom_tags ?? [],
				memberCount: counts?.memberCount,
			});
			synced++;
		}
	}
	helpers.logger.info({synced, total: guildIds.length}, 'Discovery index sync completed');
};

export default syncDiscoveryIndex;
