// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {listSeenLocales} from '../../gif/GifFeaturedCategoriesCache';
import {getGifService} from '../../middleware/ServiceSingletons';
import {getWorkerDependencies} from '../WorkerContext';

const enqueueGifFeaturedCategoriesRefresh: WorkerTaskHandler = async (_payload, helpers) => {
	const {cacheService, workerService} = getWorkerDependencies();
	const gifService = getGifService();
	for (const provider of gifService.listProviders()) {
		if (!(await provider.isAvailable())) continue;
		const locales = await listSeenLocales(cacheService, provider.meta.name);
		if (locales.length === 0) {
			helpers.logger.debug({provider: provider.meta.name}, 'No seen locales for GIF provider; nothing to refresh');
			continue;
		}
		helpers.logger.debug(
			{provider: provider.meta.name, count: locales.length},
			'Fanning out enriched GIF categories refresh',
		);
		for (const {locale, country} of locales) {
			try {
				await workerService.addJob('refreshGifFeaturedCategories', {
					provider: provider.meta.name,
					locale,
					country,
				});
			} catch (error) {
				helpers.logger.warn(
					{err: error, provider: provider.meta.name, locale, country},
					'Failed to enqueue per-locale GIF categories refresh',
				);
			}
		}
	}
};

export default enqueueGifFeaturedCategoriesRefresh;
