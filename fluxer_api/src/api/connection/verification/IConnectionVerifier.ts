// SPDX-License-Identifier: AGPL-3.0-or-later

export interface ConnectionVerificationParams {
	identifier: string;
	verification_token: string;
}

export interface IConnectionVerifier {
	verify(params: ConnectionVerificationParams): Promise<boolean>;
}
