// SPDX-License-Identifier: AGPL-3.0-or-later

import type {GifCategoryTagResponse} from '@fluxer/schema/src/domains/gif/GifSchemas';
import type {ICacheService} from '@pkgs/cache/src/ICacheService';
import {ms} from 'itty-time';

const ENRICHED_CATEGORIES_FRESH_MS = ms('30 minutes');
const ENRICHED_CATEGORIES_TTL_SECONDS = Math.floor(ms('6 hours') / 1000);
const SEEN_LOCALES_TTL_SECONDS = Math.floor(ms('7 days') / 1000);
export const REFRESH_LOCK_TTL_SECONDS = Math.floor(ms('5 minutes') / 1000);

interface EnrichedCategoriesEntry {
	data: Array<GifCategoryTagResponse>;
	timestamp: number;
}

function enrichedCategoriesCacheKey(provider: string, locale: string, country: string): string {
	return `gif:featured_categories_enriched:${provider}:${locale}:${country}`;
}

function seenLocalesSetKey(provider: string): string {
	return `gif:featured_categories_seen_locales:${provider}`;
}

function encodeLocaleMember(locale: string, country: string): string {
	return `${locale}|${country}`;
}

function decodeLocaleMember(member: string): {
	locale: string;
	country: string;
} | null {
	const idx = member.indexOf('|');
	if (idx <= 0) return null;
	const locale = member.slice(0, idx);
	const country = member.slice(idx + 1);
	if (!locale || !country) return null;
	return {locale, country};
}

export function refreshLockKey(provider: string, locale: string, country: string): string {
	return `gif:featured_categories_refresh_lock:${provider}:${locale}:${country}`;
}

export async function readEnrichedCategoriesCache(
	cache: ICacheService,
	provider: string,
	locale: string,
	country: string,
): Promise<{
	data: Array<GifCategoryTagResponse>;
	isStale: boolean;
} | null> {
	const entry = await cache.get<EnrichedCategoriesEntry>(enrichedCategoriesCacheKey(provider, locale, country));
	if (!entry) return null;
	const isStale = Date.now() - entry.timestamp > ENRICHED_CATEGORIES_FRESH_MS;
	return {data: entry.data, isStale};
}

export async function writeEnrichedCategoriesCache(
	cache: ICacheService,
	provider: string,
	locale: string,
	country: string,
	data: Array<GifCategoryTagResponse>,
): Promise<void> {
	const entry: EnrichedCategoriesEntry = {data, timestamp: Date.now()};
	await cache.set(enrichedCategoriesCacheKey(provider, locale, country), entry, ENRICHED_CATEGORIES_TTL_SECONDS);
}

export async function trackSeenLocale(
	cache: ICacheService,
	provider: string,
	locale: string,
	country: string,
): Promise<void> {
	await cache.sadd(seenLocalesSetKey(provider), encodeLocaleMember(locale, country), SEEN_LOCALES_TTL_SECONDS);
}

export async function listSeenLocales(
	cache: ICacheService,
	provider: string,
): Promise<
	Array<{
		locale: string;
		country: string;
	}>
> {
	const members = await cache.smembers(seenLocalesSetKey(provider));
	const out: Array<{
		locale: string;
		country: string;
	}> = [];
	for (const member of members) {
		const decoded = decodeLocaleMember(member);
		if (decoded) out.push(decoded);
	}
	return out;
}
