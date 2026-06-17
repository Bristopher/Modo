// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomInt, randomUUID} from 'node:crypto';
import {parseIpAddress} from '@fluxer/ip_utils/src/IpAddress';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import {Logger} from '../Logger';

const ONIONOO_URL =
	'https://onionoo.torproject.org/details?type=relay&running=true&flag=Exit&fields=exit_addresses,or_addresses';
const FETCH_TIMEOUT_MS = 30000;
const REFRESH_INTERVAL_MS = 30 * 60 * 1000;
const REFRESH_JITTER_MS = 2 * 60 * 1000;
const OPPORTUNISTIC_REFRESH_MIN_MS = 60000;
const OPPORTUNISTIC_REFRESH_MAX_MS = 3 * 60000;
const KV_PAYLOAD_KEY = 'fluxer:tor_exit_list:payload';
const KV_LOCK_KEY = 'fluxer:tor_exit_list:lock';
const LOCK_TTL_SECONDS = 5 * 60;
const PAYLOAD_TTL_SECONDS = 60 * 60;
const HYDRATE_WAIT_MS = 60000;
const HYDRATE_POLL_MIN_MS = 500;
const HYDRATE_POLL_MAX_MS = 5000;

type FetchResult =
	| {
			kind: 'fetched';
			payload: string;
	  }
	| {
			kind: 'not_modified';
	  }
	| {
			kind: 'failed';
	  };

class TorExitListCache {
	private ipv4Exits: ReadonlySet<string> = new Set();
	private ipv6Exits: ReadonlySet<string> = new Set();
	private kvClient: IKVProvider | null = null;
	private refreshTimer: NodeJS.Timeout | null = null;
	private initPromise: Promise<void> | null = null;
	private lastModified: string | null = null;
	private inflightFetch: AbortController | null = null;

	setKvClient(kv: IKVProvider | null): void {
		this.kvClient = kv;
	}

	initialize(): Promise<void> {
		if (!this.initPromise) {
			this.initPromise = this.doInitialize().catch((err) => {
				this.initPromise = null;
				throw err;
			});
		}
		return this.initPromise;
	}

	shutdown(): void {
		if (this.refreshTimer) {
			clearTimeout(this.refreshTimer);
			this.refreshTimer = null;
		}
		if (this.inflightFetch) {
			this.inflightFetch.abort();
			this.inflightFetch = null;
		}
	}

	isTorExit(ip: string): boolean {
		const parsed = parseIpAddress(ip);
		if (!parsed) return false;
		return parsed.family === 'ipv4' ? this.ipv4Exits.has(parsed.normalized) : this.ipv6Exits.has(parsed.normalized);
	}

	async forceRefresh(): Promise<void> {
		await this.tryRefresh();
	}

	seedForTesting(ips: Iterable<string>): void {
		this.applyPayload(Array.from(ips).join('\n'));
	}

	clearForTesting(): void {
		this.ipv4Exits = new Set();
		this.ipv6Exits = new Set();
	}

	private async doInitialize(): Promise<void> {
		const cached = await this.readPayloadFromKv();
		if (cached) {
			this.applyPayload(cached);
			Logger.info({ipv4Count: this.ipv4Exits.size, ipv6Count: this.ipv6Exits.size}, 'Tor exit list hydrated from KV');
			this.scheduleNextRefresh(this.opportunisticRefreshDelayMs());
			return;
		}
		await this.hydrateFromTorOrWait();
		this.scheduleNextRefresh(REFRESH_INTERVAL_MS);
	}

	private scheduleNextRefresh(baseDelayMs: number): void {
		if (this.refreshTimer) {
			clearTimeout(this.refreshTimer);
			this.refreshTimer = null;
		}
		const jitter = randomInt(REFRESH_JITTER_MS * 2) - REFRESH_JITTER_MS;
		const delay = Math.max(1000, baseDelayMs + jitter);
		this.refreshTimer = setTimeout(() => {
			this.refreshTimer = null;
			this.tryRefresh()
				.catch((err) => {
					Logger.warn({error: err instanceof Error ? err.message : String(err)}, 'Tor exit list refresh failed');
				})
				.finally(() => {
					this.scheduleNextRefresh(REFRESH_INTERVAL_MS);
				});
		}, delay);
		this.refreshTimer.unref?.();
	}

	private opportunisticRefreshDelayMs(): number {
		const range = OPPORTUNISTIC_REFRESH_MAX_MS - OPPORTUNISTIC_REFRESH_MIN_MS;
		return OPPORTUNISTIC_REFRESH_MIN_MS + randomInt(range);
	}

	private async tryRefresh(): Promise<void> {
		const kv = this.kvClient;
		if (!kv) return;
		await this.withKvLock(kv, async (acquired) => {
			if (!acquired) {
				const cached = await this.readPayloadFromKv();
				if (cached) this.applyPayload(cached);
				return;
			}
			const result = await this.fetchFromTor();
			if (result.kind !== 'fetched') return;
			await kv.setex(KV_PAYLOAD_KEY, PAYLOAD_TTL_SECONDS, result.payload).catch((err) => {
				Logger.warn(
					{error: err instanceof Error ? err.message : String(err)},
					'Failed to write Tor exit list payload to KV',
				);
			});
			this.applyPayload(result.payload);
			Logger.info({ipv4Count: this.ipv4Exits.size, ipv6Count: this.ipv6Exits.size}, 'Tor exit list refreshed');
		});
	}

	private async hydrateFromTorOrWait(): Promise<void> {
		const kv = this.kvClient;
		if (!kv) {
			const result = await this.fetchFromTor();
			if (result.kind === 'fetched') {
				this.applyPayload(result.payload);
				return;
			}
			Logger.warn('Tor exit list bootstrap could not fetch; continuing with empty list');
			return;
		}
		const fetchedAsWinner = await this.withKvLock(kv, async (acquired) => {
			if (!acquired) return false;
			const result = await this.fetchFromTor();
			if (result.kind !== 'fetched') {
				Logger.warn('Tor exit list bootstrap could not fetch; continuing with empty list');
				return true;
			}
			await kv.setex(KV_PAYLOAD_KEY, PAYLOAD_TTL_SECONDS, result.payload).catch(() => undefined);
			this.applyPayload(result.payload);
			Logger.info(
				{ipv4Count: this.ipv4Exits.size, ipv6Count: this.ipv6Exits.size},
				'Tor exit list fetched and hydrated on startup',
			);
			return true;
		});
		if (fetchedAsWinner) return;
		const cached = await this.pollForPeerPayload();
		if (cached) {
			this.applyPayload(cached);
			Logger.info(
				{ipv4Count: this.ipv4Exits.size, ipv6Count: this.ipv6Exits.size},
				'Tor exit list hydrated from KV after peer fetch',
			);
			return;
		}
		Logger.warn('Tor exit list not available via KV after wait; fetching directly');
		const result = await this.fetchFromTor();
		if (result.kind !== 'fetched') {
			Logger.warn('Tor exit list bootstrap could not fetch; continuing with empty list');
			return;
		}
		this.applyPayload(result.payload);
	}

	private async pollForPeerPayload(): Promise<string | null> {
		const deadline = Date.now() + HYDRATE_WAIT_MS;
		let backoff = HYDRATE_POLL_MIN_MS;
		while (Date.now() < deadline) {
			await sleep(backoff);
			const cached = await this.readPayloadFromKv();
			if (cached) return cached;
			backoff = Math.min(backoff * 2, HYDRATE_POLL_MAX_MS);
		}
		return null;
	}

	private async withKvLock<T>(kv: IKVProvider, fn: (acquired: boolean) => Promise<T>): Promise<T> {
		const lockToken = randomUUID();
		const acquired = await kv.acquireLock(KV_LOCK_KEY, lockToken, LOCK_TTL_SECONDS).catch(() => false);
		try {
			return await fn(acquired);
		} finally {
			if (acquired) {
				await kv.releaseLock(KV_LOCK_KEY, lockToken).catch((err) => {
					Logger.warn(
						{error: err instanceof Error ? err.message : String(err)},
						'Failed to release Tor exit list KV lock (will expire on TTL)',
					);
				});
			}
		}
	}

	private async readPayloadFromKv(): Promise<string | null> {
		const kv = this.kvClient;
		if (!kv) return null;
		try {
			return await kv.get(KV_PAYLOAD_KEY);
		} catch (err) {
			Logger.warn({error: err instanceof Error ? err.message : String(err)}, 'Failed reading Tor exit list from KV');
			return null;
		}
	}

	private async fetchFromTor(): Promise<FetchResult> {
		const controller = new AbortController();
		const timeout = setTimeout(() => controller.abort(), FETCH_TIMEOUT_MS);
		this.inflightFetch = controller;
		try {
			const headers: Record<string, string> = {
				Accept: 'application/json',
				'Accept-Encoding': 'gzip',
			};
			if (this.lastModified) {
				headers['If-Modified-Since'] = this.lastModified;
			}
			const res = await fetch(ONIONOO_URL, {
				signal: controller.signal,
				headers,
			});
			if (res.status === 304) {
				return {kind: 'not_modified'};
			}
			if (!res.ok) {
				Logger.warn({status: res.status}, 'Onionoo fetch returned non-OK status');
				return {kind: 'failed'};
			}
			const body = (await res.json()) as {
				relays?: ReadonlyArray<{
					exit_addresses?: ReadonlyArray<string>;
					or_addresses?: ReadonlyArray<string>;
				}>;
			};
			const ips = new Set<string>();
			for (const relay of body.relays ?? []) {
				for (const addr of relay.exit_addresses ?? []) {
					const ip = stripHostPort(addr);
					if (ip) ips.add(ip);
				}
				for (const addr of relay.or_addresses ?? []) {
					const ip = stripHostPort(addr);
					if (ip) ips.add(ip);
				}
			}
			const lastModified = res.headers.get('last-modified');
			if (lastModified) this.lastModified = lastModified;
			return {kind: 'fetched', payload: Array.from(ips).join('\n')};
		} catch (err) {
			const name = (
				err as
					| {
							name?: string;
					  }
					| undefined
			)?.name;
			if (name === 'AbortError') {
				Logger.warn('Onionoo fetch aborted (timeout or shutdown)');
			} else {
				Logger.warn({error: err instanceof Error ? err.message : String(err)}, 'Onionoo fetch failed');
			}
			return {kind: 'failed'};
		} finally {
			clearTimeout(timeout);
			if (this.inflightFetch === controller) {
				this.inflightFetch = null;
			}
		}
	}

	private applyPayload(payload: string): void {
		const ipv4 = new Set<string>();
		const ipv6 = new Set<string>();
		for (const rawLine of payload.split('\n')) {
			const value = rawLine.trim();
			if (!value) continue;
			const parsed = parseIpAddress(value);
			if (!parsed) continue;
			if (parsed.family === 'ipv4') {
				ipv4.add(parsed.normalized);
			} else {
				ipv6.add(parsed.normalized);
			}
		}
		this.ipv4Exits = ipv4;
		this.ipv6Exits = ipv6;
	}
}

function sleep(ms: number): Promise<void> {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

function stripHostPort(value: string): string | null {
	const trimmed = value.trim();
	if (!trimmed) return null;
	if (trimmed.startsWith('[')) {
		const end = trimmed.indexOf(']');
		if (end <= 1) return null;
		return trimmed.slice(1, end);
	}
	const firstColon = trimmed.indexOf(':');
	if (firstColon > 0 && firstColon === trimmed.lastIndexOf(':')) {
		return trimmed.slice(0, firstColon);
	}
	return trimmed || null;
}

export const torExitListCache = new TorExitListCache();
