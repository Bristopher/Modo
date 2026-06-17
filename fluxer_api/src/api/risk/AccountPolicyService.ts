// SPDX-License-Identifier: AGPL-3.0-or-later

import {Config} from '../Config';
import {
	type AccountPolicyContactCapability,
	createAccountPolicyContactContext,
	createAccountPolicyContactDomainContext,
	createAccountPolicyEvaluatorFromConfig,
	type IAccountPolicyEvaluator,
	isAccountPolicyConfigMissing,
} from './AccountPolicyEvaluator';
import type {ReverseDnsClassification} from './RiskTypes';

let injectedAccountPolicyEvaluator: IAccountPolicyEvaluator | null = null;
let configuredAccountPolicyEvaluator: IAccountPolicyEvaluator | null = null;

export function setInjectedAccountPolicyEvaluator(evaluator: IAccountPolicyEvaluator | undefined): void {
	injectedAccountPolicyEvaluator = evaluator ?? null;
	configuredAccountPolicyEvaluator = null;
}

function canRunWithoutAccountPolicyConfig(): boolean {
	return Config.nodeEnv === 'development' || Config.dev.testModeEnabled || Config.instance.selfHosted;
}

export function getAccountPolicyEvaluator(): IAccountPolicyEvaluator {
	if (injectedAccountPolicyEvaluator) return injectedAccountPolicyEvaluator;
	if (!configuredAccountPolicyEvaluator) {
		const rawPolicy = Config.risk.accountPolicyDsl;
		if (isAccountPolicyConfigMissing(rawPolicy) && !canRunWithoutAccountPolicyConfig()) {
			throw new Error('FLUXER_ACCOUNT_POLICY_DSL is required in hosted production');
		}
		configuredAccountPolicyEvaluator = createAccountPolicyEvaluatorFromConfig(rawPolicy);
	}
	return configuredAccountPolicyEvaluator;
}

export function accountPolicyContactHasCapability(
	email: string | null | undefined,
	capability: AccountPolicyContactCapability,
): boolean {
	return getAccountPolicyEvaluator()
		.evaluateContact(createAccountPolicyContactContext(email))
		.hasCapability(capability);
}

export function accountPolicyContactDomainHasCapability(
	domain: string | null | undefined,
	capability: AccountPolicyContactCapability,
): boolean {
	return getAccountPolicyEvaluator()
		.evaluateContact(createAccountPolicyContactDomainContext(domain))
		.hasCapability(capability);
}

export function isAccountPolicyContactDomainReputationExempt(domain: string | null | undefined): boolean {
	return accountPolicyContactDomainHasCapability(domain, 'reputation_checks_exempt');
}

export function classifyAccountPolicyEmailTld(tld: string | null | undefined): 'high' | null {
	return getAccountPolicyEvaluator().classifyEmailTld(tld);
}

export function isAccountPolicyLowRiskEmailTld(tld: string | null | undefined): boolean {
	return getAccountPolicyEvaluator().isLowRiskEmailTld(tld);
}

export function isAccountPolicyBlockedRegistrationEmailDomain(domain: string | null | undefined): boolean {
	return getAccountPolicyEvaluator().isBlockedRegistrationEmailDomain(domain);
}

export function isAccountPolicyTrustedCommercialPrivacyProvider(providerName: string | null | undefined): boolean {
	return getAccountPolicyEvaluator().isTrustedCommercialPrivacyProvider(providerName);
}

export function isAccountPolicyEducationOrganizationName(organizationName: string | null | undefined): boolean {
	return getAccountPolicyEvaluator().isEducationOrganizationName(organizationName);
}

export function classifyAccountPolicyReverseDnsHostname(hostname: string | null | undefined): ReverseDnsClassification {
	return getAccountPolicyEvaluator().classifyReverseDnsHostname(hostname);
}

export function getAccountPolicyMinimumAgeForRegion(
	countryCode: string | null | undefined,
	defaultAge: number,
): number {
	return getAccountPolicyEvaluator().getMinimumAgeForRegion(countryCode, defaultAge);
}
