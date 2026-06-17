// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {ensureSessionStarted} from '../../message/tests/MessageTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createFriendship, createGroupDmChannel, type GroupDmChannelResponse} from './ChannelTestUtils';

describe('Group DM name update', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	afterAll(async () => {
		await harness?.shutdown();
	});
	it('updates group DM name correctly', async () => {
		const user1 = await createTestAccount(harness);
		const user2 = await createTestAccount(harness);
		const user3 = await createTestAccount(harness);
		await ensureSessionStarted(harness, user1.token);
		await ensureSessionStarted(harness, user2.token);
		await ensureSessionStarted(harness, user3.token);
		await createFriendship(harness, user1, user2);
		await createFriendship(harness, user1, user3);
		const groupDm = await createGroupDmChannel(harness, user1.token, [user2.userId, user3.userId]);
		const updated = await createBuilder<GroupDmChannelResponse>(harness, user1.token)
			.patch(`/channels/${groupDm.id}`)
			.body({name: 'Cool Group Chat'})
			.expect(HTTP_STATUS.OK)
			.execute();
		expect(updated.name).toBe('Cool Group Chat');
	});
});
