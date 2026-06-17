// SPDX-License-Identifier: AGPL-3.0-or-later

import {FeatureTemporarilyDisabledError} from '@fluxer/errors/src/domains/core/FeatureTemporarilyDisabledError';
import {Logger} from '../Logger';
import type {IGifProvider} from './IGifProvider';

export class GifService {
	private readonly providersByName: Map<string, IGifProvider>;
	private readonly activeName: string | (() => Promise<string>);

	constructor(params: {
		providers: ReadonlyArray<IGifProvider>;
		activeName: string | (() => Promise<string>);
	}) {
		this.providersByName = new Map(params.providers.map((p) => [p.meta.name, p]));
		this.activeName = params.activeName;
		if (typeof params.activeName === 'string' && !this.providersByName.has(params.activeName)) {
			Logger.warn(
				{activeName: params.activeName, registered: Array.from(this.providersByName.keys())},
				'Active GIF provider is not registered; /gifs requests will fail until configuration is fixed',
			);
		}
	}

	async getActive(): Promise<IGifProvider> {
		const activeName = await this.getActiveName();
		const provider = this.providersByName.get(activeName);
		if (!provider || !(await provider.isAvailable())) {
			Logger.debug({activeName}, 'Active GIF provider unavailable');
			throw new FeatureTemporarilyDisabledError();
		}
		return provider;
	}

	async getActiveName(): Promise<string> {
		return typeof this.activeName === 'string' ? this.activeName : this.activeName();
	}

	getByName(name: string): IGifProvider | null {
		return this.providersByName.get(name) ?? null;
	}

	listProviders(): ReadonlyArray<IGifProvider> {
		return Array.from(this.providersByName.values());
	}
}
