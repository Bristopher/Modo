// SPDX-License-Identifier: AGPL-3.0-or-later

import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import type {
	GuildEmojiResponse,
	GuildEmojiWithUserResponse,
	GuildStickerResponse,
	GuildStickerWithUserResponse,
} from '@fluxer/schema/src/domains/guild/GuildEmojiSchemas';
import type {GuildResponse} from '@fluxer/schema/src/domains/guild/GuildResponseSchemas';
import type {PackDashboardResponse, PackSummaryResponse} from '@fluxer/schema/src/domains/pack/PackSchemas';
import {createTestAccount, type TestAccount} from '../../auth/tests/AuthTestUtils';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder, createBuilderWithoutAuth} from '../../test/TestRequestBuilder';

interface PackCreateRequest {
	name: string;
	description?: string | null;
}

interface PackUpdateRequest {
	name?: string;
	description?: string | null;
}

type PackType = 'emoji' | 'sticker';

function loadPackFixture(filename: string): Buffer {
	const fixturesPath = join(import.meta.dirname, '..', '..', 'test', 'fixtures', filename);
	return readFileSync(fixturesPath);
}

async function createGuild(harness: ApiTestHarness, token: string, name: string): Promise<GuildResponse> {
	return createBuilder<GuildResponse>(harness, token).post('/guilds').body({name}).expect(HTTP_STATUS.OK).execute();
}

export async function grantStaffAccess(harness: ApiTestHarness, userId: string): Promise<void> {
	await createBuilderWithoutAuth(harness).patch(`/test/users/${userId}/flags`).body({flags: 1}).execute();
}

export async function grantPremium(harness: ApiTestHarness, userId: string): Promise<void> {
	const premiumUntil = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString();
	await createBuilderWithoutAuth(harness)
		.post(`/test/users/${userId}/premium`)
		.body({
			premium_type: 2,
			premium_until: premiumUntil,
		})
		.execute();
}

export async function revokePremium(harness: ApiTestHarness, userId: string): Promise<void> {
	await createBuilderWithoutAuth(harness)
		.post(`/test/users/${userId}/premium`)
		.body({
			premium_type: null,
			premium_until: null,
		})
		.execute();
}

export async function setupPackTestAccount(harness: ApiTestHarness): Promise<{
	account: TestAccount;
	guild: GuildResponse;
}> {
	const account = await createTestAccount(harness);
	const guild = await createGuild(harness, account.token, 'Pack Test Guild');
	await grantStaffAccess(harness, account.userId);
	await grantPremium(harness, account.userId);
	return {account, guild};
}

export async function setupNonPremiumPackTestAccount(harness: ApiTestHarness): Promise<{
	account: TestAccount;
	guild: GuildResponse;
}> {
	const account = await createTestAccount(harness);
	const guild = await createGuild(harness, account.token, 'Pack Test Guild');
	await grantStaffAccess(harness, account.userId);
	return {account, guild};
}

export async function listPacks(harness: ApiTestHarness, token: string): Promise<PackDashboardResponse> {
	return createBuilder<PackDashboardResponse>(harness, token).get('/packs').expect(HTTP_STATUS.OK).execute();
}

export async function createPack(
	harness: ApiTestHarness,
	token: string,
	packType: PackType,
	data: PackCreateRequest,
): Promise<PackSummaryResponse> {
	return createBuilder<PackSummaryResponse>(harness, token)
		.post(`/packs/${packType}`)
		.body(data)
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function updatePack(
	harness: ApiTestHarness,
	token: string,
	packId: string,
	data: PackUpdateRequest,
): Promise<PackSummaryResponse> {
	return createBuilder<PackSummaryResponse>(harness, token)
		.patch(`/packs/${packId}`)
		.body(data)
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function deletePack(harness: ApiTestHarness, token: string, packId: string): Promise<void> {
	await createBuilder<void>(harness, token).delete(`/packs/${packId}`).expect(HTTP_STATUS.NO_CONTENT).execute();
}

export async function installPack(harness: ApiTestHarness, token: string, packId: string): Promise<void> {
	await createBuilder<void>(harness, token)
		.post(`/packs/${packId}/install`)
		.body({})
		.expect(HTTP_STATUS.NO_CONTENT)
		.execute();
}

export async function uninstallPack(harness: ApiTestHarness, token: string, packId: string): Promise<void> {
	await createBuilder<void>(harness, token).delete(`/packs/${packId}/install`).expect(HTTP_STATUS.NO_CONTENT).execute();
}

export async function getPackEmojis(
	harness: ApiTestHarness,
	token: string,
	packId: string,
): Promise<Array<GuildEmojiWithUserResponse>> {
	return createBuilder<Array<GuildEmojiWithUserResponse>>(harness, token)
		.get(`/packs/emojis/${packId}`)
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function getPackStickers(
	harness: ApiTestHarness,
	token: string,
	packId: string,
): Promise<Array<GuildStickerWithUserResponse>> {
	return createBuilder<Array<GuildStickerWithUserResponse>>(harness, token)
		.get(`/packs/stickers/${packId}`)
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function createPackEmoji(
	harness: ApiTestHarness,
	token: string,
	packId: string,
	name: string,
	imageBase64?: string,
): Promise<GuildEmojiResponse> {
	const image = imageBase64 ?? loadPackFixture('yeah.png').toString('base64');
	return createBuilder<GuildEmojiResponse>(harness, token)
		.post(`/packs/emojis/${packId}`)
		.body({name, image: `data:image/png;base64,${image}`})
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function deletePackEmoji(
	harness: ApiTestHarness,
	token: string,
	packId: string,
	emojiId: string,
): Promise<void> {
	await createBuilder<void>(harness, token)
		.delete(`/packs/emojis/${packId}/${emojiId}`)
		.expect(HTTP_STATUS.NO_CONTENT)
		.execute();
}

export async function createPackSticker(
	harness: ApiTestHarness,
	token: string,
	packId: string,
	name: string,
	tags: Array<string>,
	imageBase64?: string,
): Promise<GuildStickerResponse> {
	const image = imageBase64 ?? loadPackFixture('sticker.png').toString('base64');
	return createBuilder<GuildStickerResponse>(harness, token)
		.post(`/packs/stickers/${packId}`)
		.body({name, tags, image: `data:image/png;base64,${image}`})
		.expect(HTTP_STATUS.OK)
		.execute();
}

export async function deletePackSticker(
	harness: ApiTestHarness,
	token: string,
	packId: string,
	stickerId: string,
): Promise<void> {
	await createBuilder<void>(harness, token)
		.delete(`/packs/stickers/${packId}/${stickerId}`)
		.expect(HTTP_STATUS.NO_CONTENT)
		.execute();
}
