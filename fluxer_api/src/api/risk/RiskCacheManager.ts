// SPDX-License-Identifier: AGPL-3.0-or-later

interface MutableRef<T> {
	current: T;
}

export type ReadonlyRiskCacheRef<T> = {
	readonly current: T;
};

interface RiskCacheManagerLogger {
	info(payload: object, msg: string): void;
	warn(payload: object, msg: string): void;
}

export interface RiskCacheLoaders {
	loadDisposableDomains: () => Promise<ReadonlySet<string>>;
}

interface RiskCacheManagerOptions extends RiskCacheLoaders {
	logger?: RiskCacheManagerLogger;
}

interface RiskCacheRefreshResult {
	disposableDomainCount: number;
	subtaskErrors: ReadonlyArray<{
		step: string;
		error: string;
	}>;
}

export class RiskCacheManager {
	private readonly _disposable: MutableRef<ReadonlySet<string>> = {current: new Set()};
	readonly disposableDomainsRef: ReadonlyRiskCacheRef<ReadonlySet<string>> = this._disposable;
	private readonly logger: RiskCacheManagerLogger | undefined;
	private readonly loaders: RiskCacheLoaders;

	constructor(opts: RiskCacheManagerOptions) {
		this.logger = opts.logger;
		this.loaders = {
			loadDisposableDomains: opts.loadDisposableDomains,
		};
	}

	async refresh(): Promise<RiskCacheRefreshResult> {
		const subtaskErrors: Array<{
			step: string;
			error: string;
		}> = [];
		try {
			const set = await this.loaders.loadDisposableDomains();
			this._disposable.current = set;
			this.logger?.info({count: set.size}, 'RiskCacheManager: loaded disposable domains from DB');
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			this.logger?.warn({step: 'disposable_domains', err: message}, 'RiskCacheManager: subtask failed');
			subtaskErrors.push({step: 'disposable_domains', error: message});
		}
		return {
			disposableDomainCount: this._disposable.current.size,
			subtaskErrors,
		};
	}
}
