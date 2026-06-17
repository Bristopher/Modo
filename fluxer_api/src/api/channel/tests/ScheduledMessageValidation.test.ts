// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {createGuild} from '../../guild/tests/GuildTestUtils';
import {ensureSessionStarted} from '../../message/tests/MessageTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {generateFutureTimestamp, generatePastTimestamp, HTTP_STATUS, TEST_LIMITS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createGuildChannel, grantStaffAccess, scheduleMessage} from './ScheduledMessageTestUtils';

describe('Scheduled message validation', () => {
	let harness: ApiTestHarness;
	beforeAll(async () => {
		harness = await createApiTestHarness();
	});
	beforeEach(async () => {
		await harness.reset();
	});
	it('rejects scheduling message with past time', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'sched-validation-past');
		await grantStaffAccess(harness, owner.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'test');
		await ensureSessionStarted(harness, owner.token);
		const pastTime = generatePastTimestamp(1);
		const {json} = await createBuilder<{
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		}>(harness, owner.token)
			.post(`/channels/${channel.id}/messages/schedule`)
			.body({
				content: 'should fail - past time',
				scheduled_local_at: pastTime,
				timezone: 'UTC',
			})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.executeWithResponse();
		const errorJson = json as {
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		};
		const hasScheduledAtError = errorJson.errors.some((e) => e.path === 'scheduled_local_at');
		expect(hasScheduledAtError).toBe(true);
	});
	it('rejects scheduling message exceeding 30 days', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'sched-validation-30day');
		await grantStaffAccess(harness, owner.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'test');
		await ensureSessionStarted(harness, owner.token);
		const futureTime = new Date(Date.now() + TEST_LIMITS.SCHEDULED_MESSAGE_MAX_DELAY_MS).toISOString();
		const {json} = await createBuilder<{
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		}>(harness, owner.token)
			.post(`/channels/${channel.id}/messages/schedule`)
			.body({
				content: 'should fail - exceeds 30 days',
				scheduled_local_at: futureTime,
				timezone: 'UTC',
			})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.executeWithResponse();
		const errorJson = json as {
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		};
		const hasScheduledAtError = errorJson.errors.some((e) => e.path === 'scheduled_local_at');
		expect(hasScheduledAtError).toBe(true);
	});
	it('rejects scheduling message with invalid timezone', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'sched-validation-tz');
		await grantStaffAccess(harness, owner.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'test');
		await ensureSessionStarted(harness, owner.token);
		const futureTime = generateFutureTimestamp(5);
		const {json} = await createBuilder<{
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		}>(harness, owner.token)
			.post(`/channels/${channel.id}/messages/schedule`)
			.body({
				content: 'should fail - invalid timezone',
				scheduled_local_at: futureTime,
				timezone: 'Invalid/NotATimezone',
			})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.executeWithResponse();
		const errorJson = json as {
			errors: Array<{
				path: string;
				message: string;
				code: string;
			}>;
		};
		const hasTimezoneError = errorJson.errors.some((e) => e.path === 'timezone');
		expect(hasTimezoneError).toBe(true);
	});
	it('accepts scheduling message at 30 day boundary', async () => {
		const owner = await createTestAccount(harness);
		const guild = await createGuild(harness, owner.token, 'sched-validation-boundary');
		await grantStaffAccess(harness, owner.userId);
		const channel = await createGuildChannel(harness, owner.token, guild.id, 'test');
		const futureTime = new Date(Date.now() + 29 * 24 * 60 * 60 * 1000 + 23 * 60 * 60 * 1000).toISOString();
		const scheduled = await scheduleMessage(
			harness,
			channel.id,
			owner.token,
			'should succeed - within 30 days',
			new Date(futureTime),
		);
		expect(scheduled.status).toBe('pending');
	});
});
