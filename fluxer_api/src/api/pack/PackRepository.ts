// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../BrandedTypes';
import {BatchBuilder, fetchMany, fetchOne, upsertOne} from '../database/CassandraQueryExecution';
import {nextVersion} from '../database/CassandraTypes';
import {buildPatchFromData, executeVersionedUpdate} from '../database/CassandraVersionedUpdate';
import type {ExpressionPackRow, PackInstallationRow} from '../database/types/UserTypes';
import {EXPRESSION_PACK_COLUMNS} from '../database/types/UserTypes';
import {ExpressionPack} from '../models/ExpressionPack';
import {ExpressionPacks, ExpressionPacksByCreator, PackInstallations} from '../Tables';

export type PackType = ExpressionPack['type'];

const FETCH_EXPRESSION_PACK_BY_ID_QUERY = ExpressionPacks.select({
	where: ExpressionPacks.where.eq('pack_id'),
	limit: 1,
});
const FETCH_EXPRESSION_PACKS_BY_CREATOR_QUERY = ExpressionPacksByCreator.select({
	where: ExpressionPacksByCreator.where.eq('creator_id'),
});
const FETCH_PACK_INSTALLATIONS_BY_USER_QUERY = PackInstallations.select({
	where: PackInstallations.where.eq('user_id'),
});
const FETCH_PACK_INSTALLATION_QUERY = PackInstallations.select({
	where: [PackInstallations.where.eq('user_id'), PackInstallations.where.eq('pack_id')],
	limit: 1,
});

export class PackRepository {
	async getPack(packId: GuildID): Promise<ExpressionPack | null> {
		const row = await fetchOne<ExpressionPackRow>(FETCH_EXPRESSION_PACK_BY_ID_QUERY.bind({pack_id: packId}));
		return row ? new ExpressionPack(row) : null;
	}

	async listPacksByCreator(creatorId: UserID, packType?: PackType): Promise<Array<ExpressionPack>> {
		const rows = await fetchMany<ExpressionPackRow>(
			FETCH_EXPRESSION_PACKS_BY_CREATOR_QUERY.bind({creator_id: creatorId}),
		);
		return rows.filter((row) => (packType ? row.pack_type === packType : true)).map((row) => new ExpressionPack(row));
	}

	async countPacksByCreator(creatorId: UserID, packType: PackType): Promise<number> {
		const rows = await fetchMany<ExpressionPackRow>(
			FETCH_EXPRESSION_PACKS_BY_CREATOR_QUERY.bind({creator_id: creatorId}),
		);
		return rows.filter((row) => row.pack_type === packType).length;
	}

	async upsertPack(data: ExpressionPackRow): Promise<ExpressionPack> {
		const packId = data.pack_id;
		const previousPack = await this.getPack(packId);
		const result = await executeVersionedUpdate<ExpressionPackRow, 'pack_id'>(
			async () => {
				const existing = await fetchOne<ExpressionPackRow>(FETCH_EXPRESSION_PACK_BY_ID_QUERY.bind({pack_id: packId}));
				return existing ?? null;
			},
			(current) => ({
				pk: {pack_id: packId},
				patch: buildPatchFromData(data, current, EXPRESSION_PACK_COLUMNS, ['pack_id']),
			}),
			ExpressionPacks,
		);
		const batch = new BatchBuilder();
		if (previousPack && previousPack.creatorId !== data.creator_id) {
			batch.addPrepared(
				ExpressionPacksByCreator.deleteByPk({
					creator_id: previousPack.creatorId,
					pack_id: packId,
				}),
			);
		}
		const finalPack = new ExpressionPack({...data, version: result.finalVersion ?? nextVersion(previousPack?.version)});
		batch.addPrepared(ExpressionPacksByCreator.insert(finalPack.toRow()));
		await batch.execute();
		return finalPack;
	}

	async deletePack(packId: GuildID): Promise<void> {
		const pack = await this.getPack(packId);
		const batch = new BatchBuilder().addPrepared(ExpressionPacks.deleteByPk({pack_id: packId}));
		if (pack) {
			batch.addPrepared(
				ExpressionPacksByCreator.deleteByPk({
					creator_id: pack.creatorId,
					pack_id: packId,
				}),
			);
		}
		await batch.execute();
	}

	async listInstallations(userId: UserID): Promise<Array<PackInstallationRow>> {
		return await fetchMany<PackInstallationRow>(FETCH_PACK_INSTALLATIONS_BY_USER_QUERY.bind({user_id: userId}));
	}

	async addInstallation(data: PackInstallationRow): Promise<void> {
		await upsertOne(PackInstallations.insert(data));
	}

	async removeInstallation(userId: UserID, packId: GuildID): Promise<void> {
		await PackInstallations.deleteByPk({
			user_id: userId,
			pack_id: packId,
		});
	}

	async hasInstallation(userId: UserID, packId: GuildID): Promise<boolean> {
		const row = await fetchOne<PackInstallationRow>(
			FETCH_PACK_INSTALLATION_QUERY.bind({
				user_id: userId,
				pack_id: packId,
			}),
		);
		return row !== null;
	}
}
