// SPDX-License-Identifier: AGPL-3.0-or-later

export type ElasticsearchFilter = Record<string, unknown>;

interface ElasticsearchRangeOptions {
	gte?: number;
	lte?: number;
	gt?: number;
	lt?: number;
}

export function esTermFilter(field: string, value: string | number | boolean): ElasticsearchFilter {
	return {term: {[field]: value}};
}

export function esTermsFilter(field: string, values: Array<string | number | boolean>): ElasticsearchFilter {
	return {terms: {[field]: values}};
}

export function esRangeFilter(field: string, opts: ElasticsearchRangeOptions): ElasticsearchFilter {
	return {range: {[field]: opts}};
}

export function esExistsFilter(field: string): ElasticsearchFilter {
	return {exists: {field}};
}

export function esNotExistsFilter(field: string): ElasticsearchFilter {
	return {bool: {must_not: [{exists: {field}}]}};
}

function esMustNotTerm(field: string, value: string | number | boolean): ElasticsearchFilter {
	return {bool: {must_not: [{term: {[field]: value}}]}};
}

export function esAndTerms(field: string, values: Array<string | number | boolean>): Array<ElasticsearchFilter> {
	return values.map((v) => esTermFilter(field, v));
}

export function esExcludeAny(field: string, values: Array<string | number | boolean>): Array<ElasticsearchFilter> {
	return values.map((v) => esMustNotTerm(field, v));
}

export function compactFilters(filters: Array<ElasticsearchFilter | undefined>): Array<ElasticsearchFilter> {
	return filters.filter((f): f is ElasticsearchFilter => f !== undefined);
}
