// SPDX-License-Identifier: AGPL-3.0-or-later

import * as fs from 'node:fs';
import * as path from 'node:path';
import {fileURLToPath} from 'node:url';
import {describe, expect, test} from 'vitest';

interface OpenAPIOperation {
	operationId?: string;
	security?: Array<Record<string, Array<string>>>;
}

interface OpenAPISpec {
	paths: Record<string, Record<string, OpenAPIOperation>>;
}

const SPEC_PATH = path.join(path.dirname(fileURLToPath(import.meta.url)), '../../openapi/openapi.json');

function readPublicOpenAPISpec(): OpenAPISpec {
	return JSON.parse(fs.readFileSync(SPEC_PATH, 'utf-8')) as OpenAPISpec;
}

describe('OAuth2 OpenAPI scope surface', () => {
	test('documents exactly the routes that accept OAuth2 bearer tokens', () => {
		const spec = readPublicOpenAPISpec();
		const actual: Array<{
			method: string;
			path: string;
			operationId: string;
			oauth2: Array<Array<string>>;
		}> = [];
		const methods = ['get', 'post', 'put', 'patch', 'delete'] as const;
		for (const [routePath, pathItem] of Object.entries(spec.paths)) {
			for (const method of methods) {
				const operation = pathItem[method];
				if (!operation) continue;
				const oauth2 = (operation.security ?? [])
					.filter((entry) => entry.oauth2Token !== undefined)
					.map((entry) => [...entry.oauth2Token].sort());
				const bearer = (operation.security ?? []).filter((entry) => entry.bearerToken !== undefined);
				expect(bearer, `${method.toUpperCase()} ${routePath} should use oauth2Token, not bearerToken`).toEqual([]);
				if (oauth2.length === 0) continue;
				actual.push({
					method: method.toUpperCase(),
					path: routePath,
					operationId: operation.operationId ?? '',
					oauth2,
				});
			}
		}
		actual.sort((a, b) => `${a.method} ${a.path}`.localeCompare(`${b.method} ${b.path}`));
		expect(actual).toEqual([
			{
				method: 'GET',
				path: '/guilds/{guild_id}',
				operationId: 'get_guild',
				oauth2: [['guilds']],
			},
			{
				method: 'GET',
				path: '/guilds/{guild_id}/roles',
				operationId: 'list_guild_roles',
				oauth2: [['guilds']],
			},
			{
				method: 'GET',
				path: '/oauth2/@me',
				operationId: 'get_current_user_oauth2',
				oauth2: [[]],
			},
			{
				method: 'GET',
				path: '/oauth2/userinfo',
				operationId: 'get_oauth2_userinfo',
				oauth2: [['identify']],
			},
			{
				method: 'GET',
				path: '/users/@me',
				operationId: 'get_current_user',
				oauth2: [['identify']],
			},
			{
				method: 'GET',
				path: '/users/@me/connections',
				operationId: 'list_connections',
				oauth2: [['connections']],
			},
			{
				method: 'GET',
				path: '/users/@me/guilds',
				operationId: 'list_guilds',
				oauth2: [['guilds']],
			},
		]);
	});
});
