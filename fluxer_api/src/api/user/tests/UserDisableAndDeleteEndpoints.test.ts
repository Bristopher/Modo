// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount, loginAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {deleteAccount, disableAccount} from './UserTestUtils';

describe('User Disable And Delete Endpoints', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('disable and delete flow works correctly', async () => {
		const account = await createTestAccount(harness);
		await disableAccount(harness, account.token, account.password);
		await createBuilder(harness, account.token).get('/users/@me').expect(HTTP_STATUS.UNAUTHORIZED).execute();
		const login = await loginAccount(harness, account);
		const newToken = login.token;
		await deleteAccount(harness, newToken, account.password);
		await createBuilder(harness, newToken).get('/users/@me').expect(HTTP_STATUS.UNAUTHORIZED).execute();
		const loginAfterDelete = await loginAccount(harness, account);
		expect(loginAfterDelete.token).not.toBe('');
	});
});
