// SPDX-License-Identifier: AGPL-3.0-or-later

import {isAccountPolicyTrustedCommercialPrivacyProvider} from './AccountPolicyService';

export function isTrustedCommercialPrivacyProvider(providerName: string | null | undefined): boolean {
	return isAccountPolicyTrustedCommercialPrivacyProvider(providerName);
}
