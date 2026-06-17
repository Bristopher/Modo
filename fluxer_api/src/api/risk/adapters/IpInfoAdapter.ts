// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IpInfoService} from '@pkgs/geoip/src/IpInfoService';
import {isAccountPolicyEducationOrganizationName} from '../AccountPolicyService';
import type {IpConnectionType, IpInfoAnonymousResult} from '../RiskTypes';

interface IpInfoCheckerContext {
	ipInfoService: IpInfoService;
}

export function createIpInfoChecker(ctx: IpInfoCheckerContext) {
	return async function checkIpInfo(ip: string): Promise<IpInfoAnonymousResult> {
		const result = await ctx.ipInfoService.lookup(ip, {
			source: 'risk.ipinfo_checker',
			reason: 'registration_risk',
		});
		const isMobile = result.flags.isMobile;
		const asnType = result.asn.type;
		const asnOrg = result.asn.name;
		return {
			ip: result.ip,
			available: result.available,
			isAnonymous: result.anonymous.isAnonymous,
			providerName: result.anonymous.providerName,
			isVpn: result.anonymous.isVpn,
			isProxy: result.anonymous.isProxy,
			isResidentialProxy: result.anonymous.isResidentialProxy,
			isTor: result.anonymous.isTor,
			isRelay: result.anonymous.isRelay,
			isHosting: result.flags.isHosting,
			isMobile,
			asnType,
			asnOrg,
			connectionType: deriveConnectionType({
				available: result.available,
				isResidentialProxy: result.anonymous.isResidentialProxy,
				isHosting: result.flags.isHosting,
				isMobile,
				asnType,
				asnOrg,
			}),
			percentDaysSeen: result.anonymous.percentDaysSeen,
			riskNote: result.riskNote,
		};
	};
}

function deriveConnectionType(args: {
	available: boolean;
	isResidentialProxy: boolean;
	isHosting: boolean;
	isMobile: boolean;
	asnType: string | null;
	asnOrg: string | null;
}): IpConnectionType {
	if (!args.available) return 'unknown';
	if (args.isResidentialProxy) return 'residential_proxy';
	const asnType = args.asnType?.toLowerCase() ?? null;
	if (args.isHosting || asnType === 'hosting') return 'data_center';
	if (args.isMobile || asnType === 'mobile') return 'mobile';
	if (asnType === 'education' || matchesEducationOrg(args.asnOrg)) {
		return 'education';
	}
	if (asnType === 'business' || asnType === 'corporate') return 'corporate';
	return 'residential';
}

function matchesEducationOrg(org: string | null): boolean {
	return isAccountPolicyEducationOrganizationName(org);
}
