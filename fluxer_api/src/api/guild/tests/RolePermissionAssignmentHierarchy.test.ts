// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {acceptInvite, createChannelInvite, createGuild, createRole, getChannel, updateRole} from './GuildTestUtils';

describe('Role Permission Assignment Hierarchy', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('should prevent users from granting permissions they do not possess', async () => {
		const owner = await createTestAccount(harness);
		const manager = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'Role Hierarchy Guild');
		const roleHigh = await createRole(harness, owner.token, guild.id, {
			name: 'High',
			permissions: String(1 << 28),
		});
		const roleMid = await createRole(harness, owner.token, guild.id, {
			name: 'Mid',
			permissions: String(1 << 11),
		});
		const systemChannel = await getChannel(harness, owner.token, guild.system_channel_id!);
		const invite = await createChannelInvite(harness, owner.token, systemChannel.id);
		await acceptInvite(harness, manager.token, invite.code);
		await createBuilder(harness, owner.token)
			.put(`/guilds/${guild.id}/members/${manager.userId}/roles/${roleMid.id}`)
			.expect(HTTP_STATUS.NO_CONTENT)
			.execute();
		await createBuilder(harness, manager.token)
			.patch(`/guilds/${guild.id}/roles/${roleHigh.id}`)
			.body({permissions: String(1 << 28)})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
		await createBuilder(harness, manager.token)
			.patch(`/guilds/${guild.id}/roles/${roleMid.id}`)
			.body({permissions: String(1 << 28)})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
		const updatedRole = await updateRole(harness, owner.token, guild.id, roleMid.id, {
			permissions: String((1 << 11) | (1 << 13)),
		});
		expect(updatedRole.permissions).toBe(String((1 << 11) | (1 << 13)));
	});
});
