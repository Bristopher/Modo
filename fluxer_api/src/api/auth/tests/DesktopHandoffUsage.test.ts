// SPDX-License-Identifier: AGPL-3.0-or-later

import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilderWithoutAuth} from '../../test/TestRequestBuilder';
import {createAuthHarness, createTestAccount, loginAccount} from './AuthTestUtils';

interface HandoffInitiateResponse {
	code: string;
}

interface HandoffStatusResponse {
	status: 'pending' | 'completed' | 'expired';
	token?: string;
	user_id?: string;
}

describe('Auth desktop handoff complete single use', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createAuthHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	afterAll(async () => {
		await harness?.shutdown();
	});
	it('prevents reuse of handoff code after completion', async () => {
		const account = await createTestAccount(harness);
		const login = await loginAccount(harness, account);
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.body(null)
			.execute();
		expect(initResp.code).toBeTruthy();
		await createBuilderWithoutAuth(harness).get(`/auth/handoff/${initResp.code}/info`).execute();
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.header('Authorization', login.token)
			.body({
				code: initResp.code,
				user_id: login.userId,
			})
			.expect(204)
			.execute();
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.header('Authorization', login.token)
			.body({
				code: initResp.code,
				user_id: login.userId,
			})
			.expect(400)
			.execute();
		const status = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(status.status).toBe('completed');
		expect(status.token).toBeTruthy();
		expect(status.token).not.toBe(login.token);
	});
});
