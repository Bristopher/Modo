// SPDX-License-Identifier: AGPL-3.0-or-later

import {PremiumFlags, SuspiciousActivityFlags, UserPremiumTypes} from '@fluxer/constants/src/UserConstants';
import {afterEach, beforeEach, describe, expect, it} from 'vitest';
import type {User} from '../models/User';
import {setInjectedAccountPolicyEvaluator} from '../risk/AccountPolicyService';
import {
	createCurrentBehaviorTestAccountPolicyEvaluator,
	TEST_POLICY_CONTACT_DOMAIN,
	TEST_POLICY_CONTACT_SUBDOMAIN,
} from '../test/AccountPolicyTestEvaluator';
import {checkIsPremium, getEffectivePremiumUntil, getEffectiveSuspiciousFlags, getRequiredActions} from './UserHelpers';

function createUser(
	overrides: Partial<Pick<User, 'email' | 'emailVerified' | 'hasVerifiedPhone' | 'suspiciousActivityFlags'>> = {},
): User {
	return {
		email: 'user@fluxer.app',
		emailVerified: false,
		hasVerifiedPhone: false,
		suspiciousActivityFlags: 0,
		...overrides,
	} as User;
}

describe('getRequiredActions', () => {
	beforeEach(() => {
		setInjectedAccountPolicyEvaluator(createCurrentBehaviorTestAccountPolicyEvaluator());
	});
	afterEach(() => {
		setInjectedAccountPolicyEvaluator(undefined);
	});
	it('keeps verified-email requirements active when the account email is unverified', () => {
		const user = createUser({
			suspiciousActivityFlags: SuspiciousActivityFlags.REQUIRE_VERIFIED_EMAIL,
		});
		expect(getRequiredActions(user)).toEqual(['REQUIRE_VERIFIED_EMAIL']);
		expect(getEffectiveSuspiciousFlags(user)).toBe(SuspiciousActivityFlags.REQUIRE_VERIFIED_EMAIL);
	});
	it('masks verified-email requirements when the account has a verified email', () => {
		const user = createUser({
			emailVerified: true,
			suspiciousActivityFlags:
				SuspiciousActivityFlags.REQUIRE_VERIFIED_EMAIL |
				SuspiciousActivityFlags.REQUIRE_VERIFIED_EMAIL_OR_VERIFIED_PHONE,
		});
		expect(getRequiredActions(user)).toEqual([]);
		expect(getEffectiveSuspiciousFlags(user)).toBe(0);
	});
	it('drops weaker redundant clauses while preserving the stronger canonical action', () => {
		const user = createUser({
			suspiciousActivityFlags:
				SuspiciousActivityFlags.REQUIRE_VERIFIED_EMAIL_OR_VERIFIED_PHONE |
				SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL_OR_VERIFIED_PHONE,
		});
		expect(getRequiredActions(user)).toEqual(['REQUIRE_REVERIFIED_EMAIL_OR_VERIFIED_PHONE']);
		expect(getEffectiveSuspiciousFlags(user)).toBe(SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL_OR_VERIFIED_PHONE);
	});
	it('retains incomparable combinations so the client can complete them sequentially', () => {
		const user = createUser({
			suspiciousActivityFlags:
				SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL | SuspiciousActivityFlags.REQUIRE_REVERIFIED_PHONE,
		});
		expect(getRequiredActions(user)).toEqual(['REQUIRE_REVERIFIED_EMAIL', 'REQUIRE_REVERIFIED_PHONE']);
		expect(getEffectiveSuspiciousFlags(user)).toBe(
			SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL | SuspiciousActivityFlags.REQUIRE_REVERIFIED_PHONE,
		);
	});
	it('masks verified-phone requirements after the stored phone has been removed', () => {
		const user = createUser({
			hasVerifiedPhone: true,
			suspiciousActivityFlags: SuspiciousActivityFlags.REQUIRE_VERIFIED_PHONE,
		});
		expect(getRequiredActions(user)).toEqual([]);
		expect(getEffectiveSuspiciousFlags(user)).toBe(0);
	});
	it('masks all suspicious activity requirements for policy-exempt contact domains', () => {
		const user = createUser({
			email: `builder@${TEST_POLICY_CONTACT_DOMAIN}`,
			suspiciousActivityFlags:
				SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL |
				SuspiciousActivityFlags.REQUIRE_REVERIFIED_PHONE |
				SuspiciousActivityFlags.REQUIRE_INBOUND_PHONE_VERIFICATION,
		});
		expect(getRequiredActions(user)).toEqual([]);
		expect(getEffectiveSuspiciousFlags(user)).toBe(0);
	});
	it('does not mask suspicious activity requirements for non-matching subdomains', () => {
		const user = createUser({
			email: `builder@${TEST_POLICY_CONTACT_SUBDOMAIN}`,
			suspiciousActivityFlags: SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL,
		});
		expect(getRequiredActions(user)).toEqual(['REQUIRE_REVERIFIED_EMAIL']);
		expect(getEffectiveSuspiciousFlags(user)).toBe(SuspiciousActivityFlags.REQUIRE_REVERIFIED_EMAIL);
	});
});

describe('checkIsPremium', () => {
	it('uses the later gift extension as the effective premium end', () => {
		const premiumUntil = new Date(Date.now() + 60_000);
		const premiumGiftExtensionEndsAt = new Date(Date.now() + 120_000);
		expect(getEffectivePremiumUntil({premiumUntil, premiumGiftExtensionEndsAt})?.toISOString()).toBe(
			premiumGiftExtensionEndsAt.toISOString(),
		);
	});
	it('treats a future gift extension as active after the subscription period ended', () => {
		const user = {
			isBot: false,
			premiumType: UserPremiumTypes.SUBSCRIPTION,
			premiumUntil: new Date(Date.now() - 60_000),
			premiumGiftExtensionEndsAt: new Date(Date.now() + 60_000),
			premiumGraceEndsAt: null,
			premiumWillCancel: false,
			flags: 0n,
			premiumFlags: 0,
		};
		expect(checkIsPremium(user)).toBe(true);
	});
	it('lets the perks-disabled flag override an active paid subscription', () => {
		const user = {
			isBot: false,
			premiumType: UserPremiumTypes.SUBSCRIPTION,
			premiumUntil: new Date(Date.now() + 60_000),
			premiumGiftExtensionEndsAt: null,
			premiumGraceEndsAt: null,
			premiumWillCancel: false,
			flags: 0n,
			premiumFlags: PremiumFlags.PERKS_DISABLED,
		};
		expect(checkIsPremium(user)).toBe(false);
	});
	it('lets the perks-disabled flag override backend premium override', () => {
		const user = {
			isBot: false,
			premiumType: UserPremiumTypes.NONE,
			premiumUntil: null,
			premiumGiftExtensionEndsAt: null,
			premiumGraceEndsAt: null,
			premiumWillCancel: false,
			flags: 0n,
			premiumFlags: PremiumFlags.ENABLED_OVERRIDE | PremiumFlags.PERKS_DISABLED,
		};
		expect(checkIsPremium(user)).toBe(false);
	});
});
