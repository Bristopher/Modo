// SPDX-License-Identifier: AGPL-3.0-or-later

import {HttpResponse, http} from 'msw';

const BUNNY_EDGE_IPV4_LIST = ['198.51.100.10', '198.51.100.11'].join('\n');
const BUNNY_EDGE_IPV6_LIST = ['2001:db8::10', '2001:db8::11'].join('\n');

export function createBunnyEdgeHandlers() {
	return [
		http.get(
			'https://bunnycdn.com/api/system/edgeserverlist/plain',
			() =>
				new HttpResponse(BUNNY_EDGE_IPV4_LIST, {
					status: 200,
					headers: {'content-type': 'text/plain; charset=utf-8'},
				}),
		),
		http.get(
			'https://bunnycdn.com/api/system/edgeserverlist/IPv6/plain',
			() =>
				new HttpResponse(BUNNY_EDGE_IPV6_LIST, {
					status: 200,
					headers: {'content-type': 'text/plain; charset=utf-8'},
				}),
		),
	];
}
