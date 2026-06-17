// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildBanResponse} from '@fluxer/schema/src/domains/guild/GuildMemberSchemas';
import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {createUserID} from '../../BrandedTypes';
import {fetchMany} from '../../database/CassandraQueryExecution';
import type {RelationshipRow} from '../../database/types/UserTypes';
import {createGuild} from '../../guild/tests/GuildTestUtils';
import {RelationshipsByTarget} from '../../Tables';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createFriendship} from './RelationshipTestUtils';
import {deleteAccount, setPendingDeletionAt, triggerDeletionWorker, waitForDeletionCompletion} from './UserTestUtils';

describe('Account Delete Permanent', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('permanent account deletion removes user data', async () => {
		const account = await createTestAccount(harness);
		const friend = await createTestAccount(harness);
		await createFriendship(harness, account, friend);
		const guild = await createGuild(harness, friend.token, 'Deletion cleanup test');
		await createBuilder(harness, friend.token)
			.put(`/guilds/${guild.id}/bans/${account.userId}`)
			.body({})
			.expect(HTTP_STATUS.NO_CONTENT)
			.execute();
		const bansBefore = await createBuilder<Array<GuildBanResponse>>(harness, friend.token)
			.get(`/guilds/${guild.id}/bans`)
			.expect(HTTP_STATUS.OK)
			.execute();
		expect(bansBefore.some((ban) => ban.user.id === account.userId)).toBe(true);
		await deleteAccount(harness, account.token, account.password);
		await createBuilder(harness, account.token).get('/users/@me').expect(HTTP_STATUS.UNAUTHORIZED).execute();
		const past = new Date();
		past.setMinutes(past.getMinutes() - 1);
		await setPendingDeletionAt(harness, account.userId, past);
		await triggerDeletionWorker(harness);
		await waitForDeletionCompletion(harness, account.userId);
		const bansAfter = await createBuilder<Array<GuildBanResponse>>(harness, friend.token)
			.get(`/guilds/${guild.id}/bans`)
			.expect(HTTP_STATUS.OK)
			.execute();
		expect(bansAfter.some((ban) => ban.user.id === account.userId)).toBe(false);
		const targetRows = await fetchMany<RelationshipRow>(
			RelationshipsByTarget.selectCql({
				where: RelationshipsByTarget.where.eq('target_user_id'),
			}),
			{target_user_id: createUserID(BigInt(friend.userId))},
		);
		expect(targetRows.some((row) => row.source_user_id === createUserID(BigInt(account.userId)))).toBe(false);
	});
});
