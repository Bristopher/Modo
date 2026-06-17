// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder} from '../../test/TestRequestBuilder';

describe('DM creation requires friendship or mutual guild', () => {
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
	it('prevents DM creation with strangers who you do not share a guild with and are not friends with', async () => {
		const user1 = await createTestAccount(harness);
		const user2 = await createTestAccount(harness);
		await createBuilder(harness, user1.token)
			.post('/users/@me/channels')
			.body({recipient_id: user2.userId})
			.expect(400)
			.execute();
	});
});
