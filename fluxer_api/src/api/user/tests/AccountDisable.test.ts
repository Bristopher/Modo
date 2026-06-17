// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount, loginAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {disableAccount, expectDataExists} from './UserTestUtils';

describe('Account Disable', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('disabling account preserves data and allows re-login', async () => {
		const account = await createTestAccount(harness);
		await disableAccount(harness, account.token, account.password);
		await createBuilder(harness, account.token).get('/users/@me').expect(HTTP_STATUS.UNAUTHORIZED).execute();
		const login = await loginAccount(harness, account);
		expect(login.token).not.toBe('');
		const data = await expectDataExists(harness, account.userId);
		expect(data.emailCleared).toBe(false);
		expect(data.passwordCleared).toBe(false);
	});
});
