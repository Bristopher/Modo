// SPDX-License-Identifier: AGPL-3.0-or-later

import type {PackSummaryResponse} from '@fluxer/schema/src/domains/pack/PackSchemas';
import type {ExpressionPack} from '../models/ExpressionPack';

export function mapPackToSummary(pack: ExpressionPack, installedAt?: Date | null): PackSummaryResponse {
	const summary: PackSummaryResponse = {
		id: pack.id.toString(),
		name: pack.name,
		description: pack.description,
		type: pack.type,
		creator_id: pack.creatorId.toString(),
		created_at: pack.createdAt.toISOString(),
		updated_at: pack.updatedAt.toISOString(),
	};
	if (installedAt) {
		summary.installed_at = installedAt.toISOString();
	}
	return summary;
}
