// SPDX-License-Identifier: AGPL-3.0-or-later

import type {DisposableCheckResult} from '../RiskTypes';

interface DisposableDomainCheckerContext {
	disposableDomainsRef: {
		readonly current: ReadonlySet<string>;
	};
}

export function createDisposableDomainChecker(ctx: DisposableDomainCheckerContext) {
	return async function checkDomainDisposable(args: {domain: string}): Promise<DisposableCheckResult> {
		const domain = args.domain.toLowerCase().trim();
		const set = ctx.disposableDomainsRef.current;
		return {
			domain,
			isDisposable: set.has(domain),
			listSize: set.size,
		};
	};
}
