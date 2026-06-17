// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import test from 'node:test';
import webauthn from '../index.js';

test('loader resolves supported native filenames', () => {
	assert.equal(webauthn._private.nativeFileName('darwin', 'x64'), 'webauthn.darwin-x64.node');
	assert.equal(webauthn._private.nativeFileName('darwin', 'arm64'), 'webauthn.darwin-arm64.node');
	assert.equal(webauthn._private.nativeFileName('linux', 'x64'), 'webauthn.linux-x64-gnu.node');
	assert.equal(webauthn._private.nativeFileName('linux', 'arm64'), 'webauthn.linux-arm64-gnu.node');
	assert.equal(webauthn._private.nativeFileName('win32', 'x64'), 'webauthn.win32-x64-msvc.node');
	assert.equal(webauthn._private.nativeFileName('win32', 'arm64'), 'webauthn.win32-arm64-msvc.node');
	assert.equal(webauthn._private.nativeFileName('freebsd', 'x64'), null);
});

test('loader exposes the Fluxer WebAuthn surface', async () => {
	assert.equal(typeof webauthn.create, 'function');
	assert.equal(typeof webauthn.get, 'function');
	assert.equal(typeof webauthn.getBackendInfo, 'function');
	assert.equal(typeof webauthn.isSupported, 'function');
	const info = webauthn.getBackendInfo();
	const supported = await webauthn.isSupported();
	assert.equal(typeof supported, 'boolean');
	assert.equal(supported, info.supported);
	assert.equal(typeof info, 'object');
	assert.equal(typeof info.target, 'string');
	assert.equal(typeof info.backend, 'string');
	assert.equal(typeof info.supported, 'boolean');
	assert.equal(typeof info.ceremoniesImplemented, 'boolean');
});

test('normalization creates spec-shaped client data and Windows transport bits', () => {
	const challenge = Buffer.from([1, 2, 3, 4]);
	const normalized = webauthn._private.normalizeCreateOptions({
		origin: 'https://web.canary.fluxer.app/channels/@me',
		challenge,
		rp: {id: 'fluxer.app', name: 'Fluxer'},
		user: {id: Buffer.from('user'), name: 'name', displayName: 'Name'},
		pubKeyCredParams: [{type: 'public-key', alg: -7}],
		authenticatorSelection: {
			authenticatorAttachment: 'platform',
			residentKey: 'preferred',
			userVerification: 'required',
		},
		excludeCredentials: [{type: 'public-key', id: Buffer.from('cred'), transports: ['internal', 'hybrid']}],
		attestation: 'none',
	});
	assert.equal(normalized.rpId, 'fluxer.app');
	assert.equal(normalized.authenticatorAttachment, 1);
	assert.equal(normalized.userVerification, 1);
	assert.equal(normalized.preferResidentKey, true);
	assert.equal(normalized.requireResidentKey, false);
	assert.equal(normalized.excludeCredentials[0].transports, 0x10 | 0x20);
	assert.deepEqual(JSON.parse(normalized.clientDataJSON.toString('utf8')), {
		type: 'webauthn.create',
		challenge: 'AQIDBA',
		origin: 'https://web.canary.fluxer.app',
		crossOrigin: false,
	});
});
