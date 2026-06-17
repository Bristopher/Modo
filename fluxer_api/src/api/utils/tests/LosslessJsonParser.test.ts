// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {parseJsonPreservingLargeIntegers} from '../LosslessJsonParser';

describe('parseJsonPreservingLargeIntegers', () => {
	it('keeps safe integers as numbers', () => {
		const parsed = parseJsonPreservingLargeIntegers('{"id":9007199254740991}') as {
			id: unknown;
		};
		expect(parsed.id).toBe(9007199254740991);
		expect(typeof parsed.id).toBe('number');
	});
	it('converts unsafe integers to strings', () => {
		const parsed = parseJsonPreservingLargeIntegers('{"id":9007199254740992}') as {
			id: unknown;
		};
		expect(parsed.id).toBe('9007199254740992');
		expect(typeof parsed.id).toBe('string');
	});
	it('preserves floating point numbers', () => {
		const parsed = parseJsonPreservingLargeIntegers('{"took":0.062,"id":1472109478688579732}') as {
			took: unknown;
			id: unknown;
		};
		expect(parsed.took).toBe(0.062);
		expect(typeof parsed.took).toBe('number');
		expect(parsed.id).toBe('1472109478688579732');
	});
	it('does not touch numbers inside strings', () => {
		const parsed = parseJsonPreservingLargeIntegers('{"id":"1472109478688579732"}') as {
			id: unknown;
		};
		expect(parsed.id).toBe('1472109478688579732');
	});
	it('handles arrays of values', () => {
		const parsed = parseJsonPreservingLargeIntegers('{"arr":[1,1472109478688579732]}') as {
			arr: Array<unknown>;
		};
		expect(parsed.arr[0]).toBe(1);
		expect(parsed.arr[1]).toBe('1472109478688579732');
	});
});
