// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import * as FetchUtils from '../FetchUtils';

function createStream(chunks: Array<string>): ReadableStream<Uint8Array> {
	const encoder = new TextEncoder();
	let index = 0;
	return new ReadableStream<Uint8Array>({
		pull(controller) {
			if (index >= chunks.length) {
				controller.close();
				return;
			}
			controller.enqueue(encoder.encode(chunks[index]));
			index += 1;
		},
	});
}

describe('FetchUtils', () => {
	describe('streamToBufferWithLimit', () => {
		it('returns the full response body when it is within the limit', async () => {
			const body = await FetchUtils.streamToBufferWithLimit(createStream(['hello', ' world']), {
				maxBytes: 32,
				description: 'Test response',
			});
			expect(new TextDecoder().decode(body)).toBe('hello world');
		});
		it('rejects when the declared content-length exceeds the configured limit', async () => {
			await expect(
				FetchUtils.streamToBufferWithLimit(createStream(['hello']), {
					maxBytes: 4,
					headers: new Headers({'Content-Length': '5'}),
					description: 'Test response',
				}),
			).rejects.toBeInstanceOf(FetchUtils.ResponseBodyTooLargeError);
		});
		it('rejects when the streamed body grows beyond the configured limit', async () => {
			await expect(
				FetchUtils.streamToBufferWithLimit(createStream(['hello', ' world']), {
					maxBytes: 5,
					description: 'Test response',
				}),
			).rejects.toBeInstanceOf(FetchUtils.ResponseBodyTooLargeError);
		});
	});
});
