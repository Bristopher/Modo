// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {createGuild} from '../../guild/tests/GuildTestUtils';
import {ensureSessionStarted} from '../../message/tests/MessageTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createGuildChannel, grantStaffAccess, scheduleMessage} from './ScheduledMessageTestUtils';

describe('Scheduled message staff gating', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('rejects scheduling message before staff flag granted', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'scheduled-flag');
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'scheduled-channel');
		await ensureSessionStarted(harness, owner.token);
		await createBuilder(harness, owner.token)
			.post(`/channels/${channel.id}/messages/schedule`)
			.body({
				content: 'trying to schedule',
				scheduled_local_at: new Date(Date.now() + 60 * 1000).toISOString(),
				timezone: 'UTC',
			})
			.expect(403)
			.execute();
		await grantStaffAccess(harness, owner.userId);
		const scheduled = await scheduleMessage(harness, channel.id, owner.token, 'enabled now');
		expect(scheduled.id).toBeDefined();
		expect(scheduled.id).not.toBe('');
	});
});
