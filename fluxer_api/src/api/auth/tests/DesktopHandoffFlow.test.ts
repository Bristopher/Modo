// SPDX-License-Identifier: AGPL-3.0-or-later

import {APIErrorCodes} from '@fluxer/constants/src/ApiErrorCodes';
import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilderWithoutAuth} from '../../test/TestRequestBuilder';
import {createAuthHarness, createTestAccount, fetchMe, loginAccount} from './AuthTestUtils';

interface HandoffInitiateResponse {
	code: string;
	expires_at: string;
}

interface HandoffInfoResponse {
	status: 'pending' | 'expired';
	client_info?: {
		platform?: string | null;
		os?: string | null;
		location?: {
			city?: string | null;
			region?: string | null;
			country?: string | null;
		} | null;
	} | null;
}

interface HandoffStatusResponse {
	status: 'pending' | 'completed' | 'expired';
	token?: string;
	user_id?: string;
}

interface UserMeResponse {
	id: string;
	email: string | null;
	username: string;
	global_name: string | null;
}

function validateHandoffCodeFormat(code: string): boolean {
	return /^[A-Z0-9]{6}-[A-Z0-9]{6}$/.test(code);
}

describe('Auth desktop handoff flow', () => {
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
	it('completes full handoff flow: initiate → info → complete → status', async () => {
		const account = await createTestAccount(harness);
		const login = await loginAccount(harness, account);
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.body(null)
			.execute();
		expect(initResp.code).toBeTruthy();
		expect(validateHandoffCodeFormat(initResp.code)).toBe(true);
		expect(initResp.expires_at).toBeTruthy();
		const info = await createBuilderWithoutAuth<HandoffInfoResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/info`)
			.execute();
		expect(info.status).toBe('pending');
		expect(info.client_info).toBeTruthy();
		const pending = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(pending.status).toBe('pending');
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.body({
				code: initResp.code,
				token: login.token,
				user_id: login.userId,
			})
			.expect(204)
			.execute();
		const completed = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(completed.status).toBe('completed');
		expect(completed.token).toBeTruthy();
		expect(completed.token).not.toBe(login.token);
		expect(completed.user_id).toBe(login.userId);
		const originalSession = await fetchMe(harness, login.token);
		expect(originalSession.response.status).toBe(200);
		const originalUser = originalSession.json as UserMeResponse;
		expect(originalUser.id).toBe(login.userId);
		const handoffSession = await fetchMe(harness, completed.token!);
		expect(handoffSession.response.status).toBe(200);
		const handoffUser = handoffSession.json as UserMeResponse;
		expect(handoffUser.id).toBe(login.userId);
		const retrieved = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(retrieved.status).toBe('expired');
	});
	it('cancels handoff correctly', async () => {
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.body(null)
			.execute();
		await createBuilderWithoutAuth(harness).delete(`/auth/handoff/${initResp.code}`).expect(204).execute();
		const cancelled = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(cancelled.status).toBe('expired');
	});
	it('info endpoint is non-destructive (code still valid after)', async () => {
		const account = await createTestAccount(harness);
		const login = await loginAccount(harness, account);
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.body(null)
			.execute();
		await createBuilderWithoutAuth<HandoffInfoResponse>(harness).get(`/auth/handoff/${initResp.code}/info`).execute();
		await createBuilderWithoutAuth<HandoffInfoResponse>(harness).get(`/auth/handoff/${initResp.code}/info`).execute();
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.header('Authorization', login.token)
			.body({
				code: initResp.code,
				user_id: login.userId,
			})
			.expect(204)
			.execute();
		const completed = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.execute();
		expect(completed.status).toBe('completed');
		expect(completed.token).toBeTruthy();
	});
	it('rejects info lookup after 3 attempts on same code', async () => {
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.body(null)
			.execute();
		for (let i = 0; i < 3; i++) {
			const info = await createBuilderWithoutAuth<HandoffInfoResponse>(harness)
				.get(`/auth/handoff/${initResp.code}/info`)
				.execute();
			expect(info.status).toBe('pending');
		}
		await createBuilderWithoutAuth(harness)
			.get(`/auth/handoff/${initResp.code}/info`)
			.expect(HTTP_STATUS.BAD_REQUEST, APIErrorCodes.INVALID_HANDOFF_CODE)
			.execute();
	});
	it('allows handoff completion across rotated IPv6 privacy addresses on the same /64', async () => {
		const originIp = '2a01:e0a:d10:95b0:8f54:410e:f290:1c66';
		const approverInfoIp = '2a01:e0a:d10:95b0:1e4:53a8:d0dd:7733';
		const approverCompleteIp = '2a01:e0a:d10:95b0:b53f:16d3:aff2:9b0f';
		const pollerIp = '2a01:e0a:d10:95b0:5e4e:f862:62d3:d9cc';
		const account = await createTestAccount(harness, {ipAddress: originIp});
		const login = await loginAccount(harness, account);
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.header('x-forwarded-for', originIp)
			.body(null)
			.execute();
		const info = await createBuilderWithoutAuth<HandoffInfoResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/info`)
			.header('x-forwarded-for', approverInfoIp)
			.execute();
		expect(info.status).toBe('pending');
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.header('x-forwarded-for', approverCompleteIp)
			.header('Authorization', login.token)
			.body({
				code: initResp.code,
				user_id: login.userId,
			})
			.expect(204)
			.execute();
		const completed = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.header('x-forwarded-for', pollerIp)
			.execute();
		expect(completed.status).toBe('completed');
		expect(completed.user_id).toBe(login.userId);
	});
	it('allows desktop handoff across different IPv4 addresses', async () => {
		const originIp = '203.0.113.10';
		const approverInfoIp = '203.0.113.11';
		const approverCompleteIp = '203.0.113.12';
		const pollerIp = '203.0.113.13';
		const account = await createTestAccount(harness, {ipAddress: originIp});
		const login = await loginAccount(harness, account);
		const initResp = await createBuilderWithoutAuth<HandoffInitiateResponse>(harness)
			.post('/auth/handoff/initiate')
			.header('x-forwarded-for', originIp)
			.body(null)
			.execute();
		const info = await createBuilderWithoutAuth<HandoffInfoResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/info`)
			.header('x-forwarded-for', approverInfoIp)
			.execute();
		expect(info.status).toBe('pending');
		await createBuilderWithoutAuth(harness)
			.post('/auth/handoff/complete')
			.header('x-forwarded-for', approverCompleteIp)
			.header('Authorization', login.token)
			.body({
				code: initResp.code,
				user_id: login.userId,
			})
			.expect(204)
			.execute();
		const completed = await createBuilderWithoutAuth<HandoffStatusResponse>(harness)
			.get(`/auth/handoff/${initResp.code}/status`)
			.header('x-forwarded-for', pollerIp)
			.execute();
		expect(completed.status).toBe('completed');
		expect(completed.user_id).toBe(login.userId);
	});
});
