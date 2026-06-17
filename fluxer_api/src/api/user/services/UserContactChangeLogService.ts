// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../../BrandedTypes';
import type {UserContactChangeLogRow} from '../../database/types/UserTypes';
import type {User} from '../../models/User';
import type {UserContactChangeLogRepository} from '../repositories/UserContactChangeLogRepository';

type ContactChangeReason = 'user_requested' | 'admin_action';

interface RecordDiffParams {
	oldUser: User | null;
	newUser: User;
	reason: ContactChangeReason;
	actorUserId: UserID | null;
	eventAt?: Date;
}

interface ListLogsParams {
	userId: UserID;
	limit?: number;
	beforeEventId?: string;
}

export class UserContactChangeLogService {
	private readonly DEFAULT_LIMIT = 50;

	constructor(private readonly repo: UserContactChangeLogRepository) {}

	async recordDiff(params: RecordDiffParams): Promise<void> {
		const {oldUser, newUser, reason, actorUserId, eventAt} = params;
		const tasks: Array<Promise<void>> = [];
		const oldEmail = oldUser?.email?.toLowerCase() ?? null;
		const newEmail = newUser.email?.toLowerCase() ?? null;
		if (oldEmail !== newEmail) {
			tasks.push(
				this.repo.insertLog({
					userId: newUser.id,
					field: 'email',
					oldValue: oldEmail,
					newValue: newEmail,
					reason,
					actorUserId,
					eventAt,
				}),
			);
		}
		const oldHasVerifiedPhone = oldUser?.hasVerifiedPhone ?? false;
		const newHasVerifiedPhone = newUser.hasVerifiedPhone;
		if (oldHasVerifiedPhone !== newHasVerifiedPhone) {
			tasks.push(
				this.repo.insertLog({
					userId: newUser.id,
					field: 'has_verified_phone',
					oldValue: String(oldHasVerifiedPhone),
					newValue: String(newHasVerifiedPhone),
					reason,
					actorUserId,
					eventAt,
				}),
			);
		}
		const oldTag = oldUser ? this.buildFluxerTag(oldUser) : null;
		const newTag = this.buildFluxerTag(newUser);
		if (oldTag !== newTag) {
			tasks.push(
				this.repo.insertLog({
					userId: newUser.id,
					field: 'fluxer_tag',
					oldValue: oldTag,
					newValue: newTag,
					reason,
					actorUserId,
					eventAt,
				}),
			);
		}
		if (tasks.length > 0) {
			await Promise.all(tasks);
		}
	}

	async listLogs(params: ListLogsParams): Promise<Array<UserContactChangeLogRow>> {
		const {userId, beforeEventId} = params;
		const limit = params.limit ?? this.DEFAULT_LIMIT;
		return this.repo.listLogs({userId, limit, beforeEventId});
	}

	private buildFluxerTag(user: User | null): string | null {
		if (!user) return null;
		const discriminator = user.discriminator?.toString() ?? '';
		if (!user.username || discriminator === '') {
			return null;
		}
		const paddedDiscriminator = discriminator.padStart(4, '0');
		return `${user.username}#${paddedDiscriminator}`;
	}
}
