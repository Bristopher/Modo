// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GuildInviteMetadataResponse} from '@fluxer/schema/src/domains/invite/InviteSchemas';
import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createGuild, deleteInvite, getChannel} from './GuildTestUtils';

describe('Invite Validation', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('should reject getting nonexistent invite', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token).get('/invites/invalidcode123').expect(HTTP_STATUS.NOT_FOUND).execute();
	});
	it('should reject accepting nonexistent invite', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.post('/invites/invalidcode123')
			.body(null)
			.expect(HTTP_STATUS.NOT_FOUND)
			.execute();
	});
	it('should reject deleting nonexistent invite', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.delete('/invites/invalidcode123')
			.expect(HTTP_STATUS.NOT_FOUND)
			.execute();
	});
	it('should reject invalid max_uses value', async () => {
		const account = await createTestAccount(harness);
		const guild = await createGuild(harness, account.token, 'Invite Validation Guild');
		const channel = await getChannel(harness, account.token, guild.system_channel_id!);
		await createBuilder(harness, account.token)
			.post(`/channels/${channel.id}/invites`)
			.body({max_uses: -1})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	it('should reject invalid max_age value', async () => {
		const account = await createTestAccount(harness);
		const guild = await createGuild(harness, account.token, 'Invite Validation Guild');
		const channel = await getChannel(harness, account.token, guild.system_channel_id!);
		await createBuilder(harness, account.token)
			.post(`/channels/${channel.id}/invites`)
			.body({max_age: -1})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	it('should accept valid max_uses and max_age', async () => {
		const account = await createTestAccount(harness);
		const guild = await createGuild(harness, account.token, 'Invite Validation Guild');
		const channel = await getChannel(harness, account.token, guild.system_channel_id!);
		const invite = await createBuilder<GuildInviteMetadataResponse>(harness, account.token)
			.post(`/channels/${channel.id}/invites`)
			.body({max_uses: 5, max_age: 3600})
			.execute();
		expect(invite.code).toBeTruthy();
		await deleteInvite(harness, account.token, invite.code);
	});
});
