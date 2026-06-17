// SPDX-License-Identifier: AGPL-3.0-or-later

import {HttpResponse, http} from 'msw';

interface OpenNsfwMockConfig {
	nsfwProbability?: number;
}

export function createOpenNsfwHandlers(config: OpenNsfwMockConfig = {}) {
	const nsfwProbability = config.nsfwProbability ?? 0;
	return [
		http.post('http://opennsfw2.fluxer.svc.cluster.local:8000/predict/image', () => {
			return HttpResponse.json({
				nsfw_probability: nsfwProbability,
			});
		}),
		http.post('http://opennsfw2.fluxer.svc.cluster.local:8000/predict/images', async ({request}) => {
			const body = (await request.json().catch(() => null)) as {
				images?: Array<unknown>;
			} | null;
			const count = Array.isArray(body?.images) ? body.images.length : 0;
			return HttpResponse.json({
				predictions: Array.from({length: count}, () => ({
					nsfw_probability: nsfwProbability,
				})),
			});
		}),
	];
}
