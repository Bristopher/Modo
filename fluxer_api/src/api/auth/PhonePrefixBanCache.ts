// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IKVProvider, IKVSubscription} from '@pkgs/kv_client/src/IKVProvider';
import {AdminRepository} from '../admin/AdminRepository';
import {PHONE_PREFIX_BAN_REFRESH_CHANNEL} from '../constants/PhonePrefixBan';
import {Logger} from '../Logger';

const BUILT_IN_BANNED_PHONE_PREFIXES: ReadonlyArray<string> = [
	'+93',
	'+95',
	'+223',
	'+225',
	'+228',
	'+233',
	'+236',
	'+240',
	'+244',
	'+245',
	'+255',
	'+256',
	'+257',
	'+261',
	'+263',
	'+269',
	'+387',
	'+996',
];

export class PhonePrefixBanCache {
	private prefixes: Set<string> = new Set();
	private prefixLengths: ReadonlyArray<number> = [];
	private isInitialized = false;
	private adminRepository = new AdminRepository();
	private kvClient: IKVProvider | null = null;
	private kvSubscription: IKVSubscription | null = null;
	private subscriberInitialized = false;
	private messageHandler: ((channel: string) => void) | null = null;
	private consecutiveFailures = 0;
	private readonly maxConsecutiveFailures = 5;

	setRefreshSubscriber(kvClient: IKVProvider | null): void {
		this.kvClient = kvClient;
	}

	async initialize(): Promise<void> {
		if (this.isInitialized) return;
		await this.refresh();
		this.isInitialized = true;
		this.setupSubscriber();
	}

	private setupSubscriber(): void {
		if (this.subscriberInitialized || !this.kvClient) return;
		const subscription = this.kvClient.duplicate();
		this.kvSubscription = subscription;
		this.messageHandler = (channel: string) => {
			if (channel === PHONE_PREFIX_BAN_REFRESH_CHANNEL) {
				this.refresh().catch((err) => {
					this.consecutiveFailures++;
					const message = err instanceof Error ? err.message : String(err);
					if (this.consecutiveFailures >= this.maxConsecutiveFailures) {
						Logger.error({error: message}, 'Failed to refresh phone-prefix ban cache after notification');
					} else {
						Logger.warn({error: message}, 'Failed to refresh phone-prefix ban cache after notification');
					}
				});
			}
		};
		subscription
			.connect()
			.then(() => subscription.subscribe(PHONE_PREFIX_BAN_REFRESH_CHANNEL))
			.then(() => {
				if (this.messageHandler) {
					subscription.on('message', this.messageHandler);
				}
			})
			.catch((error) => {
				Logger.error({error}, 'Failed to subscribe to phone-prefix ban refresh channel');
			});
		this.subscriberInitialized = true;
	}

	async refresh(): Promise<void> {
		const list = await this.adminRepository.loadAllBannedPhonePrefixes();
		const next = new Set<string>();
		for (const p of BUILT_IN_BANNED_PHONE_PREFIXES) {
			if (p.length > 0) next.add(p);
		}
		for (const p of list) {
			if (p.length > 0) next.add(p);
		}
		this.prefixes = next;
		this.prefixLengths = this.computeSortedLengths();
		this.consecutiveFailures = 0;
	}

	isBlocked(phone: string): boolean {
		const size = this.prefixes.size;
		if (size === 0) return false;
		if (size < PhonePrefixBanCache.SMALL_TABLE_THRESHOLD) {
			for (const prefix of this.prefixes) {
				if (phone.startsWith(prefix)) return true;
			}
			return false;
		}
		const lengths = this.prefixLengths;
		const phoneLen = phone.length;
		for (let i = 0; i < lengths.length; i++) {
			const len = lengths[i]!;
			if (len > phoneLen) return false;
			if (this.prefixes.has(phone.substring(0, len))) return true;
		}
		return false;
	}

	private static readonly SMALL_TABLE_THRESHOLD = 16;

	ban(prefix: string): void {
		if (prefix.length === 0) return;
		if (this.prefixes.has(prefix)) return;
		this.prefixes.add(prefix);
		this.prefixLengths = this.computeSortedLengths();
	}

	unban(prefix: string): void {
		if (!this.prefixes.delete(prefix)) return;
		this.prefixLengths = this.computeSortedLengths();
	}

	snapshot(): ReadonlySet<string> {
		return this.prefixes;
	}

	resetForTests(): void {
		this.prefixes = new Set();
		this.prefixLengths = [];
		this.isInitialized = false;
	}

	private computeSortedLengths(): ReadonlyArray<number> {
		const seen = new Set<number>();
		for (const p of this.prefixes) seen.add(p.length);
		return [...seen].sort((a, b) => a - b);
	}

	shutdown(): void {
		if (this.kvSubscription && this.messageHandler) {
			this.kvSubscription.off('message', this.messageHandler);
		}
		if (this.kvSubscription) {
			this.kvSubscription.disconnect();
			this.kvSubscription = null;
		}
		this.messageHandler = null;
	}
}

export const phonePrefixBanCache = new PhonePrefixBanCache();
