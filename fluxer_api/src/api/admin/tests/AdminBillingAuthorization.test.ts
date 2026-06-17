// SPDX-License-Identifier: AGPL-3.0-or-later

import {beforeEach, describe, test} from 'vitest';
import {createTestAccount, setUserACLs} from '../../auth/tests/AuthTestUtils';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createApiTestHarness} from '../../test/ApiTestHarness';
import {HTTP_STATUS} from '../../test/TestConstants';
import {createBuilder} from '../../test/TestRequestBuilder';

describe('Admin Billing Authorization', () => {
	let harness: ApiTestHarness;
	beforeEach(async () => {
		harness = await createApiTestHarness();
	});
	test('billing overview requires billing:view ACL', async () => {
		const admin = await createTestAccount(harness);
		await setUserACLs(harness, admin, ['admin:authenticate']);
		await createBuilder(harness, `${admin.token}`)
			.get(`/admin/billing/users/${admin.userId}/overview`)
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
	});
	test('billing refund requires billing:refund ACL', async () => {
		const admin = await createTestAccount(harness);
		await setUserACLs(harness, admin, ['admin:authenticate']);
		await createBuilder(harness, `${admin.token}`)
			.post(`/admin/billing/users/${admin.userId}/refund`)
			.body({payment_intent_id: 'pi_test_refund'})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
	});
	test('billing subscription management requires billing:manage_subscription ACL', async () => {
		const admin = await createTestAccount(harness);
		await setUserACLs(harness, admin, ['admin:authenticate']);
		await createBuilder(harness, `${admin.token}`)
			.post(`/admin/billing/users/${admin.userId}/cancel-subscription`)
			.body({})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
		await createBuilder(harness, `${admin.token}`)
			.post(`/admin/billing/users/${admin.userId}/reactivate-subscription`)
			.body({})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
	});
	test('refund policy immediate cancellation requires both refund and subscription management ACLs', async () => {
		const refundOnlyAdmin = await createTestAccount(harness);
		await setUserACLs(harness, refundOnlyAdmin, ['admin:authenticate', 'billing:refund']);
		await createBuilder(harness, `${refundOnlyAdmin.token}`)
			.post(`/admin/billing/users/${refundOnlyAdmin.userId}/refund-policy-cancel-now`)
			.body({})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
		const subscriptionOnlyAdmin = await createTestAccount(harness);
		await setUserACLs(harness, subscriptionOnlyAdmin, ['admin:authenticate', 'billing:manage_subscription']);
		await createBuilder(harness, `${subscriptionOnlyAdmin.token}`)
			.post(`/admin/billing/users/${subscriptionOnlyAdmin.userId}/refund-policy-cancel-now`)
			.body({})
			.expect(HTTP_STATUS.FORBIDDEN)
			.execute();
	});
});
