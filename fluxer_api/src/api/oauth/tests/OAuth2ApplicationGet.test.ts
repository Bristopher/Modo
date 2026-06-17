// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder, createBuilderWithoutAuth} from '../../test/TestRequestBuilder';
import {createOAuth2Application, createUniqueApplicationName, getOAuth2Application} from './OAuth2TestUtils';

describe('OAuth2 Application Get', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('returns application response shape', async () => {
		const account = await createTestAccount(harness);
		const appName = createUniqueApplicationName();
		const redirectURIs = ['https://example.com/callback', 'https://example.com/callback2'];
		const createResult = await createOAuth2Application(harness, account.token, {
			name: appName,
			redirect_uris: redirectURIs,
		});
		const application = await getOAuth2Application(harness, account.token, createResult.application.id);
		expect(application.id).toBeTruthy();
		expect(application.name).toBe(appName);
		expect(application.redirect_uris).toEqual(redirectURIs);
		expect(application.bot).toBeDefined();
		expect(application.bot?.id).toBeTruthy();
		expect(application.bot?.username).toBeTruthy();
		expect(application.bot?.discriminator).toBeTruthy();
		expect(application.bot?.token).toBeUndefined();
		expect(application.client_secret).toBeUndefined();
	});
	test('returns 404 for non-existent application', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.get('/oauth2/applications/999999999999999999')
			.expect(HTTP_STATUS.NOT_FOUND)
			.execute();
	});
	test('enforces access control - user cannot access another users application', async () => {
		const owner = await createTestAccount(harness);
		const otherUser = await createTestAccount(harness);
		const createResult = await createOAuth2Application(harness, owner.token, {
			name: createUniqueApplicationName(),
		});
		await createBuilder(harness, otherUser.token)
			.get(`/oauth2/applications/${createResult.application.id}`)
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
	});
	test('requires authentication', async () => {
		await createBuilderWithoutAuth(harness).get('/oauth2/applications/123').expect(HTTP_STATUS.UNAUTHORIZED).execute();
	});
});
