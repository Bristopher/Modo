// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, it} from 'vitest';
import {createTestBotAccount} from '../../bot/tests/BotTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';

describe('Theme bot user denied', () => {
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
	it('rejects theme creation from bot users', async () => {
		const botAccount = await createTestBotAccount(harness);
		await createBuilder(harness, `Bot ${botAccount.botToken}`)
			.post('/users/@me/themes')
			.body({css: '.test { color: red; }'})
			.expect(HTTP_STATUS.FORBIDDEN, 'ACCESS_DENIED')
			.execute();
	});
});
