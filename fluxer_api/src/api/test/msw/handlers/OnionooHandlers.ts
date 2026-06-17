// SPDX-License-Identifier: AGPL-3.0-or-later

import {HttpResponse, http} from 'msw';

export function createOnionooDetailsHandler() {
	return http.get('https://onionoo.torproject.org/details', () => {
		return HttpResponse.json(
			{relays: []},
			{
				status: 200,
				headers: {
					'last-modified': 'Mon, 01 Jan 2024 00:00:00 GMT',
				},
			},
		);
	});
}
