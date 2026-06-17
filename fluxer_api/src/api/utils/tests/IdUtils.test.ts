// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import {toIdString, toSortedIdArray} from '../IdUtils';

describe('toIdString', () => {
	it('returns null for null input', () => {
		expect(toIdString(null)).toBeNull();
	});
	it('returns null for undefined input', () => {
		expect(toIdString(undefined)).toBeNull();
	});
	it('converts bigint to string', () => {
		expect(toIdString(123456789012345678n)).toBe('123456789012345678');
	});
	it('converts string to string (passthrough)', () => {
		expect(toIdString('123456789012345678')).toBe('123456789012345678');
	});
	it('handles zero bigint', () => {
		expect(toIdString(0n)).toBe('0');
	});
	it('handles very large bigints', () => {
		const large = 999999999999999999999999999999n;
		expect(toIdString(large)).toBe(large.toString());
	});
	it('handles negative bigints', () => {
		expect(toIdString(-1n)).toBe('-1');
	});
	it('preserves numeric string exactly', () => {
		expect(toIdString('0')).toBe('0');
		expect(toIdString('00123')).toBe('00123');
	});
});

describe('toSortedIdArray', () => {
	it('returns empty array for null input', () => {
		expect(toSortedIdArray(null)).toEqual([]);
	});
	it('returns empty array for undefined input', () => {
		expect(toSortedIdArray(undefined)).toEqual([]);
	});
	it('returns empty array for empty array input', () => {
		expect(toSortedIdArray([])).toEqual([]);
	});
	it('converts and sorts array of bigints', () => {
		const input = [300n, 100n, 200n];
		expect(toSortedIdArray(input)).toEqual(['100', '200', '300']);
	});
	it('converts and sorts array of strings', () => {
		const input = ['300', '100', '200'];
		expect(toSortedIdArray(input)).toEqual(['100', '200', '300']);
	});
	it('converts Set to sorted array', () => {
		const input = new Set([300n, 100n, 200n]);
		expect(toSortedIdArray(input)).toEqual(['100', '200', '300']);
	});
	it('handles single element array', () => {
		expect(toSortedIdArray([42n])).toEqual(['42']);
	});
	it('handles already sorted input', () => {
		const input = [100n, 200n, 300n];
		expect(toSortedIdArray(input)).toEqual(['100', '200', '300']);
	});
	it('handles reverse sorted input', () => {
		const input = [300n, 200n, 100n];
		expect(toSortedIdArray(input)).toEqual(['100', '200', '300']);
	});
	it('sorts lexicographically (string sort)', () => {
		const input = [2n, 10n, 1n];
		expect(toSortedIdArray(input)).toEqual(['1', '10', '2']);
	});
	it('handles duplicates in input array', () => {
		const input = [100n, 100n, 200n];
		expect(toSortedIdArray(input)).toEqual(['100', '100', '200']);
	});
	it('handles Set with single element', () => {
		const input = new Set([123n]);
		expect(toSortedIdArray(input)).toEqual(['123']);
	});
	it('handles empty Set', () => {
		const input = new Set<bigint>();
		expect(toSortedIdArray(input)).toEqual([]);
	});
});
