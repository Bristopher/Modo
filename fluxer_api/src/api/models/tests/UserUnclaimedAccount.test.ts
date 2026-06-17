// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {createUserID} from '../../BrandedTypes';
import {EMPTY_USER_ROW, type UserRow} from '../../database/types/UserTypes';
import {User} from '../User';

function createUserRow(overrides: Partial<UserRow> = {}): UserRow {
	return {
		...EMPTY_USER_ROW,
		user_id: createUserID(1n),
		username: 'test_user',
		discriminator: 1,
		bot: false,
		password_hash: 'hash',
		traits: new Set<string>(),
		...overrides,
	};
}

describe('User.isUnclaimedAccount', () => {
	it('returns true for non-bot users without a password hash', () => {
		const user = new User(
			createUserRow({
				password_hash: null,
				traits: new Set<string>(),
			}),
		);
		expect(user.isUnclaimedAccount()).toBe(true);
	});
	it('returns false for SSO users without a password hash', () => {
		const user = new User(
			createUserRow({
				password_hash: null,
				traits: new Set<string>(['sso']),
			}),
		);
		expect(user.isUnclaimedAccount()).toBe(false);
	});
	it('returns false for bots without a password hash', () => {
		const user = new User(
			createUserRow({
				password_hash: null,
				bot: true,
			}),
		);
		expect(user.isUnclaimedAccount()).toBe(false);
	});
});
