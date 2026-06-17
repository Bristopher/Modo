// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IAdminRepository} from '../admin/IAdminRepository';
import type {RiskCacheLoaders} from './RiskCacheManager';

interface BuildRiskCacheLoadersDeps {
	adminRepository: Pick<IAdminRepository, 'listSuspiciousEmailDomains' | 'listDisposableEmailDomains'>;
}

export function buildRiskCacheLoaders(deps: BuildRiskCacheLoadersDeps): RiskCacheLoaders {
	return {
		loadDisposableDomains: async () => {
			const [suspicious, disposable] = await Promise.all([
				deps.adminRepository.listSuspiciousEmailDomains(),
				deps.adminRepository.listDisposableEmailDomains(),
			]);
			const set = new Set<string>();
			for (const d of suspicious) set.add(d.toLowerCase());
			for (const d of disposable) set.add(d.toLowerCase());
			return set;
		},
	};
}
