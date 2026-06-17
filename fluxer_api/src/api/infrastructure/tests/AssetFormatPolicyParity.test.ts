// SPDX-License-Identifier: AGPL-3.0-or-later

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {ASSET_FORMAT_POLICY} from '@fluxer/constants/src/AssetFormatPolicy';
import {describe, expect, it} from 'vitest';

const THIS_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(THIS_DIR, '../../../../..');

function parseRustAssetExtensionFunction(source: string, functionName: 'name' | 'mime'): Map<string, string> {
	const regex = new RegExp(`pub fn ${functionName}\\(self\\)[^{]+\\{\\s*match self \\{([\\s\\S]*?)\\n\\s*\\}`, 'm');
	const match = regex.exec(source);
	expect(match, `missing Rust AssetExtension.${functionName}() match block`).not.toBeNull();
	const block = match?.[1] ?? '';
	return new Map([...block.matchAll(/Self::([A-Za-z]+)\s*=>\s*"([^"]+)"/g)].map((entry) => [entry[1], entry[2]]));
}

describe('AssetFormatPolicy parity', () => {
	it('matches the media proxy extension MIME table', () => {
		const rustSource = fs.readFileSync(path.join(REPO_ROOT, 'fluxer_media_proxy/src/constants.rs'), 'utf8');
		const rustNames = parseRustAssetExtensionFunction(rustSource, 'name');
		const rustMimes = parseRustAssetExtensionFunction(rustSource, 'mime');
		const rustMimesByExtension = new Map(
			[...rustNames.entries()].map(([variantName, extension]) => [extension, rustMimes.get(variantName)]),
		);
		expect([...rustMimesByExtension.keys()]).toEqual(ASSET_FORMAT_POLICY.attachment.upload);
		expect(Object.fromEntries(rustMimesByExtension.entries())).toEqual(ASSET_FORMAT_POLICY.attachment.mimes);
	});
});
