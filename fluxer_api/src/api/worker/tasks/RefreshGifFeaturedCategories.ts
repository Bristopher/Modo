// SPDX-License-Identifier: AGPL-3.0-or-later

import type {WorkerTaskHandler} from '@pkgs/worker/src/contracts/WorkerTask';
import {getGifService} from '../../middleware/ServiceSingletons';

interface RefreshPayload {
	provider?: string;
	locale?: string;
	country?: string;
}

const refreshGifFeaturedCategories: WorkerTaskHandler = async (rawPayload, helpers) => {
	const payload = (rawPayload ?? {}) as RefreshPayload;
	const {provider, locale, country} = payload;
	if (!provider || !locale || !country) {
		helpers.logger.warn({payload}, 'refreshGifFeaturedCategories called without required fields');
		return;
	}
	const gifService = getGifService();
	const target = gifService.getByName(provider);
	if (!target) {
		helpers.logger.warn({provider}, 'refreshGifFeaturedCategories: unknown provider');
		return;
	}
	if (!(await target.isAvailable())) {
		helpers.logger.debug({provider}, 'refreshGifFeaturedCategories: provider not configured, skipping');
		return;
	}
	helpers.logger.debug({provider, locale, country}, 'Refreshing enriched GIF featured categories');
	await target.refreshFeaturedCategories({locale, country});
};

export default refreshGifFeaturedCategories;
