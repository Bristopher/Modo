// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ICacheService} from '@pkgs/cache/src/ICacheService';
import {getIpAddressReverse} from '../../utils/IpUtils';
import {classifyAccountPolicyReverseDnsHostname} from '../AccountPolicyService';
import type {ReverseDnsResult} from '../RiskTypes';

interface ReverseDnsAdapterContext {
	cacheService?: ICacheService;
	resolver?: (ip: string) => Promise<string | null>;
	timeoutMs?: number;
}

export function createReverseDnsLookup(ctx: ReverseDnsAdapterContext = {}) {
	const timeoutMs = ctx.timeoutMs ?? 500;
	return async function lookupReverseDns(ip: string): Promise<ReverseDnsResult> {
		const hostname = ctx.resolver
			? await ctx.resolver(ip)
			: await getIpAddressReverse(ip, ctx.cacheService, {timeoutMs});
		return {
			ip,
			hostname,
			classification: classifyAccountPolicyReverseDnsHostname(hostname),
		};
	};
}
