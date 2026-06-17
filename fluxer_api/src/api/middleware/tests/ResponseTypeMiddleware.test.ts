// SPDX-License-Identifier: AGPL-3.0-or-later

import {SnowflakeType} from '@fluxer/schema/src/primitives/SchemaPrimitives';
import {Hono} from 'hono';
import {describe, expect, test} from 'vitest';
import {z} from 'zod';
import type {HonoEnv} from '../../types/HonoEnv';
import {OpenAPI, ResponseType} from '../ResponseTypeMiddleware';

const SnowflakeResponse = z.object({id: SnowflakeType});

describe('ResponseTypeMiddleware', () => {
	test('serializes SnowflakeType response transforms as JSON strings', async () => {
		const app = new Hono<HonoEnv>();
		app.get('/snowflake', ResponseType(SnowflakeResponse), (ctx) => ctx.json({id: '123456789012345678'}));

		const response = await app.request('/snowflake');

		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({id: '123456789012345678'});
	});

	test('serializes OpenAPI SnowflakeType response transforms as JSON strings', async () => {
		const app = new Hono<HonoEnv>();
		app.get(
			'/snowflake',
			OpenAPI({
				operationId: 'get_snowflake_test',
				summary: 'Get snowflake test',
				description: 'Returns a snowflake-shaped ID for response serialization regression coverage.',
				responseSchema: SnowflakeResponse,
				tags: ['Tests'],
			}),
			(ctx) => ctx.json({id: '123456789012345678'}),
		);

		const response = await app.request('/snowflake');

		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({id: '123456789012345678'});
	});
});
