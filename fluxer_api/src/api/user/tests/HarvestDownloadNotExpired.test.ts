// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeEach, describe, expect, test} from 'vitest';
import {createTestAccount} from '../../auth/tests/AuthTestUtils';
import {type ApiTestHarness, createApiTestHarness} from '../../test/ApiTestHarness';
import {fetchHarvestDownload, markHarvestCompleted, requestHarvest} from './HarvestTestUtils';

describe('Harvest Download Not Expired', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('download succeeds when harvest has not expired', async () => {
		const account = await createTestAccount(harness);
		const {harvest_id} = await requestHarvest(harness, account.token);
		const validTime = new Date(Date.now() + 6 * 24 * 60 * 60 * 1000);
		await markHarvestCompleted(account.userId, harvest_id, validTime);
		const download = await fetchHarvestDownload(harness, account.token, harvest_id);
		expect(download.download_url).not.toBe('');
		expect(download.expires_at).not.toBe('');
	});
});
