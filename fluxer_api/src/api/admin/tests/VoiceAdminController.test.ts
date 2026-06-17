// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import type {
	CreateVoiceRegionResponse,
	CreateVoiceServerResponse,
	ListVoiceRegionsResponse,
	UpdateVoiceServerResponse,
} from '@fluxer/schema/src/domains/admin/AdminVoiceSchemas';
import {afterEach, beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount, setUserACLs, type TestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {VoiceRepository} from '../../voice/VoiceRepository';

interface VoiceFixture {
	regionId: string;
	serverId: string;
	initialApiKey: string;
	initialApiSecret: string;
}

async function createAdminWithAcls(harness: ApiTestHarness, acls: Array<string>): Promise<TestAccount> {
	const account = await createTestAccount(harness);
	return await setUserACLs(harness, account, [AdminACLs.AUTHENTICATE, ...acls]);
}

async function createVoiceFixture(
	harness: ApiTestHarness,
	admin: TestAccount,
	params: {
		regionId: string;
		serverId: string;
		endpoint: string;
		apiKey: string;
		apiSecret: string;
	},
): Promise<VoiceFixture> {
	await createBuilder<CreateVoiceRegionResponse>(harness, `${admin.token}`)
		.post('/admin/voice/regions/create')
		.body({
			id: params.regionId,
			name: `Region ${params.regionId}`,
			emoji: ':earth_americas:',
			latitude: 1,
			longitude: 2,
		})
		.expect(HTTP_STATUS.OK)
		.execute();
	await createBuilder<CreateVoiceServerResponse>(harness, `${admin.token}`)
		.post('/admin/voice/servers/create')
		.body({
			region_id: params.regionId,
			server_id: params.serverId,
			endpoint: params.endpoint,
			api_key: params.apiKey,
			api_secret: params.apiSecret,
		})
		.expect(HTTP_STATUS.OK)
		.execute();
	return {
		regionId: params.regionId,
		serverId: params.serverId,
		initialApiKey: params.apiKey,
		initialApiSecret: params.apiSecret,
	};
}

describe('VoiceAdminController', () => {
	let harness: ApiTestHarness;
	let voiceRepository: VoiceRepository;
	beforeEach(async () => {
		harness = await createApiTestHarness();
		voiceRepository = new VoiceRepository();
	});
	afterEach(async () => {
		await harness?.shutdown();
	});
	test('returns servers when listing voice regions with include_servers enabled', async () => {
		const admin = await createAdminWithAcls(harness, [
			AdminACLs.VOICE_REGION_CREATE,
			AdminACLs.VOICE_REGION_LIST,
			AdminACLs.VOICE_SERVER_CREATE,
		]);
		const fixture = await createVoiceFixture(harness, admin, {
			regionId: 'voice-region-list-with-servers',
			serverId: 'voice-server-list-with-servers',
			endpoint: 'https://voice-list.example.com/socket',
			apiKey: 'list-api-key',
			apiSecret: 'list-api-secret',
		});
		const result = await createBuilder<ListVoiceRegionsResponse>(harness, `${admin.token}`)
			.post('/admin/voice/regions/list')
			.body({include_servers: true})
			.expect(HTTP_STATUS.OK)
			.execute();
		expect(result.regions).toHaveLength(1);
		expect(result.regions[0]?.id).toBe(fixture.regionId);
		expect(result.regions[0]?.servers).toBeDefined();
		expect(result.regions[0]?.servers).toHaveLength(1);
		expect(result.regions[0]?.servers?.[0]?.server_id).toBe(fixture.serverId);
	});
	test('updates voice server credentials when api key and secret are provided', async () => {
		const admin = await createAdminWithAcls(harness, [
			AdminACLs.VOICE_REGION_CREATE,
			AdminACLs.VOICE_SERVER_CREATE,
			AdminACLs.VOICE_SERVER_UPDATE,
		]);
		const fixture = await createVoiceFixture(harness, admin, {
			regionId: 'voice-region-credentials-update',
			serverId: 'voice-server-credentials-update',
			endpoint: 'https://voice-original.example.com/socket',
			apiKey: 'original-api-key',
			apiSecret: 'original-api-secret',
		});
		await createBuilder<UpdateVoiceServerResponse>(harness, `${admin.token}`)
			.post('/admin/voice/servers/update')
			.body({
				region_id: fixture.regionId,
				server_id: fixture.serverId,
				endpoint: 'https://voice-updated.example.com/socket',
				api_key: 'updated-api-key',
				api_secret: 'updated-api-secret',
			})
			.expect(HTTP_STATUS.OK)
			.execute();
		const persisted = await voiceRepository.getServer(fixture.regionId, fixture.serverId);
		expect(persisted).not.toBeNull();
		expect(persisted?.endpoint).toBe('https://voice-updated.example.com/socket');
		expect(persisted?.apiKey).toBe('updated-api-key');
		expect(persisted?.apiSecret).toBe('updated-api-secret');
	});
	test('updates api key and api secret independently when only one field is provided', async () => {
		const admin = await createAdminWithAcls(harness, [
			AdminACLs.VOICE_REGION_CREATE,
			AdminACLs.VOICE_SERVER_CREATE,
			AdminACLs.VOICE_SERVER_UPDATE,
		]);
		const fixture = await createVoiceFixture(harness, admin, {
			regionId: 'voice-region-credentials-partial-update',
			serverId: 'voice-server-credentials-partial-update',
			endpoint: 'https://voice-partial-before.example.com/socket',
			apiKey: 'partial-before-api-key',
			apiSecret: 'partial-before-api-secret',
		});
		await createBuilder<UpdateVoiceServerResponse>(harness, `${admin.token}`)
			.post('/admin/voice/servers/update')
			.body({
				region_id: fixture.regionId,
				server_id: fixture.serverId,
				api_key: 'partial-after-api-key',
			})
			.expect(HTTP_STATUS.OK)
			.execute();
		const afterApiKeyUpdate = await voiceRepository.getServer(fixture.regionId, fixture.serverId);
		expect(afterApiKeyUpdate).not.toBeNull();
		expect(afterApiKeyUpdate?.apiKey).toBe('partial-after-api-key');
		expect(afterApiKeyUpdate?.apiSecret).toBe(fixture.initialApiSecret);
		await createBuilder<UpdateVoiceServerResponse>(harness, `${admin.token}`)
			.post('/admin/voice/servers/update')
			.body({
				region_id: fixture.regionId,
				server_id: fixture.serverId,
				api_secret: 'partial-after-api-secret',
			})
			.expect(HTTP_STATUS.OK)
			.execute();
		const afterApiSecretUpdate = await voiceRepository.getServer(fixture.regionId, fixture.serverId);
		expect(afterApiSecretUpdate).not.toBeNull();
		expect(afterApiSecretUpdate?.apiKey).toBe('partial-after-api-key');
		expect(afterApiSecretUpdate?.apiSecret).toBe('partial-after-api-secret');
	});
	test('keeps voice server credentials unchanged when api key and secret are omitted', async () => {
		const admin = await createAdminWithAcls(harness, [
			AdminACLs.VOICE_REGION_CREATE,
			AdminACLs.VOICE_SERVER_CREATE,
			AdminACLs.VOICE_SERVER_UPDATE,
		]);
		const fixture = await createVoiceFixture(harness, admin, {
			regionId: 'voice-region-credentials-unchanged',
			serverId: 'voice-server-credentials-unchanged',
			endpoint: 'https://voice-before.example.com/socket',
			apiKey: 'before-api-key',
			apiSecret: 'before-api-secret',
		});
		await createBuilder<UpdateVoiceServerResponse>(harness, `${admin.token}`)
			.post('/admin/voice/servers/update')
			.body({
				region_id: fixture.regionId,
				server_id: fixture.serverId,
				endpoint: 'https://voice-after.example.com/socket',
			})
			.expect(HTTP_STATUS.OK)
			.execute();
		const persisted = await voiceRepository.getServer(fixture.regionId, fixture.serverId);
		expect(persisted).not.toBeNull();
		expect(persisted?.endpoint).toBe('https://voice-after.example.com/socket');
		expect(persisted?.apiKey).toBe(fixture.initialApiKey);
		expect(persisted?.apiSecret).toBe(fixture.initialApiSecret);
	});
});
