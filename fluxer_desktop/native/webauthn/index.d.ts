// SPDX-License-Identifier: AGPL-3.0-or-later

import type {Buffer} from 'node:buffer';

export type AuthenticatorAttachment = 'platform' | 'cross-platform' | string;
export type PublicKeyCredentialType = 'public-key';
export type UserVerificationRequirement = 'discouraged' | 'preferred' | 'required';
export type ResidentKeyRequirement = 'discouraged' | 'preferred' | 'required';
export type AttestationConveyancePreference = 'direct' | 'enterprise' | 'indirect' | 'none';

export interface PublicKeyCredentialDescriptor {
	id: Buffer;
	type: PublicKeyCredentialType;
	transports?: ReadonlyArray<string>;
}

export interface PublicKeyCredentialCreationOptions {
	attestation?: AttestationConveyancePreference;
	authenticatorSelection?: {
		authenticatorAttachment?: AuthenticatorAttachment;
		requireResidentKey?: boolean;
		residentKey?: ResidentKeyRequirement;
		userVerification?: UserVerificationRequirement;
	};
	challenge: Buffer;
	excludeCredentials?: ReadonlyArray<PublicKeyCredentialDescriptor>;
	extensions?: unknown;
	pubKeyCredParams: ReadonlyArray<{
		alg: number;
		type: PublicKeyCredentialType;
	}>;
	rp: {
		id?: string;
		name: string;
	};
	timeout?: number;
	origin: string;
	user: {
		displayName: string;
		id: Buffer;
		name: string;
	};
	windowHandle?: Buffer;
	pin?: string;
}

export interface PublicKeyCredentialRequestOptions {
	allowCredentials?: ReadonlyArray<PublicKeyCredentialDescriptor>;
	challenge: Buffer;
	extensions?: unknown;
	origin: string;
	rpId?: string;
	timeout?: number;
	userVerification?: UserVerificationRequirement;
	windowHandle?: Buffer;
	pin?: string;
}

export interface PublicKeyCredential {
	authenticatorAttachment?: AuthenticatorAttachment;
	id: string;
	rawId: Buffer;
	response: Buffer;
	type: PublicKeyCredentialType;
}

export interface WebAuthnBackendInfo {
	apiVersion: number;
	backend: 'macos-authenticationservices' | 'windows-webauthn' | 'linux-libfido2' | 'unavailable';
	ceremoniesImplemented: boolean;
	nativeLoaded: boolean;
	platformAuthenticatorAvailable: boolean;
	platformBrokerAvailable: boolean;
	reason: string;
	supported: boolean;
	target: string;
}

export declare function create(options: PublicKeyCredentialCreationOptions): Promise<PublicKeyCredential>;

export declare function get(options: PublicKeyCredentialRequestOptions): Promise<PublicKeyCredential>;

export declare function getBackendInfo(): WebAuthnBackendInfo;

export declare function isSupported(): Promise<boolean>;

export declare const loadError: Error | null;
