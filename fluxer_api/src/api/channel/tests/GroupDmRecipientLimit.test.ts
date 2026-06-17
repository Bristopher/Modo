// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createFriendship, seedPrivateChannels} from './ChannelTestUtils';

const MAX_GROUP_DM_LIMIT = 150;
const MAX_GROUP_DM_ERROR_CODE = 'MAX_GROUP_DMS';

describe('Group DM Recipient Limit', () => {
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
	it('rejects creating group DM when user has reached limit', async () => {
		const creator = await createTestAccount(harness);
		const target = await createTestAccount(harness);
		const recipient = await createTestAccount(harness);
		const helper = await createTestAccount(harness);
		await createFriendship(harness, creator, target);
		await createFriendship(harness, creator, recipient);
		const seedResult = await seedPrivateChannels(harness, target.token, target.userId, {
			group_dm_count: MAX_GROUP_DM_LIMIT,
			recipients: [helper.userId, recipient.userId],
			clear_existing: true,
		});
		expect(seedResult.group_dms).toHaveLength(MAX_GROUP_DM_LIMIT);
		await createBuilder(harness, creator.token)
			.post('/users/@me/channels')
			.body({
				recipients: [helper.userId, target.userId],
			})
			.expect(HTTP_STATUS.BAD_REQUEST, MAX_GROUP_DM_ERROR_CODE)
			.execute();
	});
});
