// SPDX-License-Identifier: AGPL-3.0-or-later

import {Permissions} from '@fluxer/constants/src/ChannelConstants';
import {MAX_MESSAGE_LENGTH_PREMIUM} from '@fluxer/constants/src/LimitConstants';
import type {MessageResponse} from '@fluxer/schema/src/domains/message/MessageResponseSchemas';
import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {authorizeBot, createTestBotAccount} from '../../bot/tests/BotTestUtils';
import {ensureSessionStarted} from '../../message/tests/MessageTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {createPermissionOverwrite, setupTestGuildWithMembers} from './ChannelTestUtils';

describe('Message send permissions', () => {
	let harness: ApiTestHarness;

	beforeAll(async () => {
		harness = await createApiTestHarness();
	});

	beforeEach(async () => {
		await harness.reset();
	});

	afterAll(async () => {
		await harness?.shutdown();
	});

	it('returns the created message when the sender cannot read history or add reactions', async () => {
		const {owner, members, systemChannel} = await setupTestGuildWithMembers(harness, 1);
		const member = members[0]!;
		await createPermissionOverwrite(harness, owner.token, systemChannel.id, member.userId, {
			type: 1,
			allow: (Permissions.VIEW_CHANNEL | Permissions.SEND_MESSAGES).toString(),
			deny: (Permissions.READ_MESSAGE_HISTORY | Permissions.ADD_REACTIONS).toString(),
		});
		await ensureSessionStarted(harness, member.token);

		const sentMessage = await createBuilder<MessageResponse>(harness, member.token)
			.post(`/channels/${systemChannel.id}/messages`)
			.body({content: 'no history send'})
			.expect(HTTP_STATUS.OK)
			.execute();

		expect(sentMessage.content).toBe('no history send');
		expect(sentMessage.author.id).toBe(member.userId);
	});

	it('allows bot message content up to 4000 characters', async () => {
		const {owner, guild, systemChannel} = await setupTestGuildWithMembers(harness, 0);
		const botAccount = await createTestBotAccount(harness);
		const botPermissions = (Permissions.VIEW_CHANNEL | Permissions.SEND_MESSAGES).toString();
		await authorizeBot(harness, owner.token, botAccount.appId, ['bot'], guild.id, botPermissions);
		const content = 'b'.repeat(MAX_MESSAGE_LENGTH_PREMIUM);

		const sentMessage = await createBuilder<MessageResponse>(harness, `Bot ${botAccount.botToken}`)
			.post(`/channels/${systemChannel.id}/messages`)
			.body({content})
			.expect(HTTP_STATUS.OK)
			.execute();

		expect(sentMessage.content).toBe(content);
		expect(sentMessage.author.id).toBe(botAccount.botUserId);
	});
});
