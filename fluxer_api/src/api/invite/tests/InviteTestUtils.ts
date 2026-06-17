// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createBuilder} from '../../test/TestRequestBuilder';

export async function deleteInvite(harness: ApiTestHarness, token: string, code: string): Promise<void> {
	await createBuilder(harness, token).delete(`/invites/${code}`).expect(204).execute();
}
