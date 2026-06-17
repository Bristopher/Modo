// SPDX-License-Identifier: AGPL-3.0-or-later

import assert from 'node:assert/strict';
import test from 'node:test';
import platformInfo from '../index.js';

test('loader exposes a stable optional native surface', () => {
	assert.equal(typeof platformInfo, 'object');
	assert.equal(platformInfo.getGpuInfo === null || typeof platformInfo.getGpuInfo === 'function', true);
	assert.equal(platformInfo.loadError === null || platformInfo.loadError instanceof Error, true);
});

test('loader resolves native binaries for supported platforms and architectures', () => {
	assert.equal(platformInfo._private.nativeFileName('darwin', 'x64'), 'platform-info.darwin-x64.node');
	assert.equal(platformInfo._private.nativeFileName('darwin', 'arm64'), 'platform-info.darwin-arm64.node');
	assert.equal(platformInfo._private.nativeFileName('linux', 'x64'), 'platform-info.linux-x64-gnu.node');
	assert.equal(platformInfo._private.nativeFileName('linux', 'arm64'), 'platform-info.linux-arm64-gnu.node');
	assert.equal(platformInfo._private.nativeFileName('win32', 'x64'), 'platform-info.win32-x64-msvc.node');
	assert.equal(platformInfo._private.nativeFileName('win32', 'arm64'), 'platform-info.win32-arm64-msvc.node');
});
