// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {createGuild} from '../../guild/tests/GuildTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {
	cancelScheduledMessage,
	createGuildChannel,
	getScheduledMessages,
	grantStaffAccess,
	scheduleMessage,
} from './ScheduledMessageTestUtils';

describe('Scheduled messages list lifecycle', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('lists scheduled messages and removes after cancel', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'scheduled-list');
		await grantStaffAccess(harness, owner.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'scheduled-list');
		const content = 'list scheduled';
		const scheduled = await scheduleMessage(harness, channel.id, owner.token, content);
		const list = await getScheduledMessages(harness, owner.token);
		const found = list.some((entry) => entry.id === scheduled.id);
		expect(found).toBe(true);
		await cancelScheduledMessage(harness, scheduled.id, owner.token);
		const listAfterCancel = await getScheduledMessages(harness, owner.token);
		const foundAfterCancel = listAfterCancel.some((entry) => entry.id === scheduled.id);
		expect(foundAfterCancel).toBe(false);
	});
});
