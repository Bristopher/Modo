// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildID, UserID} from '../BrandedTypes';
import type {ExpressionPackRow} from '../database/types/UserTypes';

type ExpressionPackType = 'emoji' | 'sticker';

export class ExpressionPack {
	readonly id: GuildID;
	readonly type: ExpressionPackType;
	readonly creatorId: UserID;
	readonly name: string;
	readonly description: string | null;
	readonly createdAt: Date;
	readonly updatedAt: Date;
	readonly version: number;

	constructor(row: ExpressionPackRow) {
		this.id = row.pack_id;
		this.type = row.pack_type as ExpressionPackType;
		this.creatorId = row.creator_id;
		this.name = row.name;
		this.description = row.description ?? null;
		this.createdAt = row.created_at;
		this.updatedAt = row.updated_at;
		this.version = row.version;
	}

	toRow(): ExpressionPackRow {
		return {
			pack_id: this.id,
			pack_type: this.type,
			creator_id: this.creatorId,
			name: this.name,
			description: this.description,
			created_at: this.createdAt,
			updated_at: this.updatedAt,
			version: this.version,
		};
	}
}
