// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {createGuild} from '../../guild/tests/GuildTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {
	createChannelInvite,
	createGuildChannel,
	getScheduledMessages,
	grantStaffAccess,
	joinGuild,
	removeGuildMember,
	scheduleMessage,
	triggerScheduledMessageWorker,
} from './ScheduledMessageTestUtils';

describe('Scheduled messages list invalid entry', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('shows invalid scheduled message in list', async () => {
		const owner = await createTestAccount(harness);
		const member = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'scheduled-invalid');
		await grantStaffAccess(harness, member.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'scheduled-invalid');
		const invite = await createChannelInvite(harness, owner.token, channel.id);
		await joinGuild(harness, member.token, invite.code);
		const content = 'invalid scheduled';
		const scheduled = await scheduleMessage(harness, channel.id, member.token, content);
		await removeGuildMember(harness, owner.token, guild.id, member.userId);
		await triggerScheduledMessageWorker(harness, member.userId, scheduled.id);
		const list = await getScheduledMessages(harness, member.token);
		const entry = list.find((e) => e.id === scheduled.id);
		expect(entry).toBeDefined();
		expect(entry!.status).toBe('invalid');
		expect(entry!.status_reason).not.toBeNull();
		expect(entry!.status_reason).not.toBe('');
	});
});
