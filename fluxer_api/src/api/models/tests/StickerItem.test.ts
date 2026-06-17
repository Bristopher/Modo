// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {createStickerID} from '../../BrandedTypes';
import {StickerItem} from '../StickerItem';

describe('StickerItem', () => {
	it('omits nsfw when it is false', () => {
		const sticker = new StickerItem({
			sticker_id: createStickerID(1n),
			name: 'party-parrot',
			animated: false,
		});
		const serialized = sticker.toMessageStickerItem();
		expect(serialized).toEqual({
			sticker_id: createStickerID(1n),
			name: 'party-parrot',
			animated: false,
		});
		expect(serialized).not.toHaveProperty('nsfw');
	});
	it('preserves nsfw when it is true', () => {
		const sticker = new StickerItem({
			sticker_id: createStickerID(2n),
			name: 'spicy-parrot',
			animated: true,
			nsfw: true,
		});
		expect(sticker.toMessageStickerItem()).toEqual({
			sticker_id: createStickerID(2n),
			name: 'spicy-parrot',
			animated: true,
			nsfw: true,
		});
	});
});
