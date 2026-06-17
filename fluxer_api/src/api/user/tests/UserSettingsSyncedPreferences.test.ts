// SPDX-License-Identifier: AGPL-3.0-or-later

import {create, equals} from '@bufbuild/protobuf';
import {
	decodeSyncedPreferences,
	encodeSyncedPreferences,
	SYNCED_PREFERENCES_MAX_BYTES,
	SyncedPreferencesSchema,
} from '@fluxer/schema/src/domains/user/SyncedPreferencesCodec';
import {AccessibilitySettingsSchema} from '@fluxer/schema/src/gen/fluxer/user/preferences/v1/accessibility_pb';
import {
	LocalUserSpamOverridesSchema,
	SearchEngineSettingsSchema,
} from '@fluxer/schema/src/gen/fluxer/user/preferences/v1/preferences_pb';
import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';
import {updateUserSettings} from './UserTestUtils';

describe('User Settings synced_preferences', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('default response contains an empty string', async () => {
		const account = await createTestAccount(harness);
		const {json} = await updateUserSettings(harness, account.token, {});
		expect(json.synced_preferences).toBe('');
	});
	test('round-trips an encoded snapshot', async () => {
		const account = await createTestAccount(harness);
		const snapshot = create(SyncedPreferencesSchema, {
			searchEngines: create(SearchEngineSettingsSchema, {textSearchEngineId: 'google'}),
			accessibility: create(AccessibilitySettingsSchema, {fontSize: 14, alwaysUnderlineLinks: true}),
			sanitizeUrls: true,
		});
		const encoded = encodeSyncedPreferences(snapshot);
		const {json} = await updateUserSettings(harness, account.token, {
			synced_preferences: encoded,
		});
		expect(typeof json.synced_preferences).toBe('string');
		expect(json.synced_preferences.length).toBeGreaterThan(0);
		expect(equals(SyncedPreferencesSchema, decodeSyncedPreferences(json.synced_preferences), snapshot)).toBe(true);
	});
	test('full-snapshot replacement removes prior fields', async () => {
		const account = await createTestAccount(harness);
		const initial = create(SyncedPreferencesSchema, {
			sanitizeUrls: true,
			accessibility: create(AccessibilitySettingsSchema, {fontSize: 14}),
			searchEngines: create(SearchEngineSettingsSchema, {textSearchEngineId: 'google'}),
		});
		await updateUserSettings(harness, account.token, {
			synced_preferences: encodeSyncedPreferences(initial),
		});
		const replacement = create(SyncedPreferencesSchema, {sanitizeUrls: true});
		const {json} = await updateUserSettings(harness, account.token, {
			synced_preferences: encodeSyncedPreferences(replacement),
		});
		expect(equals(SyncedPreferencesSchema, decodeSyncedPreferences(json.synced_preferences), replacement)).toBe(true);
	});
	test('null clears the snapshot entirely', async () => {
		const account = await createTestAccount(harness);
		await updateUserSettings(harness, account.token, {
			synced_preferences: encodeSyncedPreferences(create(SyncedPreferencesSchema, {sanitizeUrls: true})),
		});
		const {json} = await updateUserSettings(harness, account.token, {
			synced_preferences: null,
		});
		expect(json.synced_preferences).toBe('');
	});
	test('rejects non-string payloads with 400', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.patch('/users/@me/settings')
			.body({synced_preferences: {a: 1}})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	test('rejects invalid base64 with 400', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.patch('/users/@me/settings')
			.body({synced_preferences: 'not_base64!!!'})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	test('rejects garbage protobuf bytes with 400', async () => {
		const account = await createTestAccount(harness);
		await createBuilder(harness, account.token)
			.patch('/users/@me/settings')
			.body({synced_preferences: '////////'})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	test('rejects oversized snapshots', async () => {
		const account = await createTestAccount(harness);
		const blob = 'x'.repeat(2048);
		const ids: Array<string> = [];
		const entries = Math.ceil(SYNCED_PREFERENCES_MAX_BYTES / blob.length) + 8;
		for (let i = 0; i < entries; i++) ids.push(`${i}-${blob}`);
		const big = create(SyncedPreferencesSchema, {
			localSpamOverrides: create(LocalUserSpamOverridesSchema, {spammerUserIds: ids}),
		});
		const encoded = encodeSyncedPreferences(big);
		await createBuilder(harness, account.token)
			.patch('/users/@me/settings')
			.body({synced_preferences: encoded})
			.expect(HTTP_STATUS.BAD_REQUEST)
			.execute();
	});
	test('omitting the field leaves the existing snapshot intact', async () => {
		const account = await createTestAccount(harness);
		const initial = encodeSyncedPreferences(
			create(SyncedPreferencesSchema, {
				searchEngines: create(SearchEngineSettingsSchema, {textSearchEngineId: 'google'}),
			}),
		);
		await updateUserSettings(harness, account.token, {synced_preferences: initial});
		const {json} = await updateUserSettings(harness, account.token, {
			gif_auto_play: false,
		});
		expect(json.synced_preferences).toBe(initial);
		expect(json.gif_auto_play).toBe(false);
	});
});
