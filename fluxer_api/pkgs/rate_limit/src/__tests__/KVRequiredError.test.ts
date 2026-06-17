// SPDX-License-Identifier: AGPL-3.0-or-later

import {throwKVRequiredError} from '@pkgs/rate_limit/src/KVRequiredError';
import {describe, expect, it} from 'vitest';

describe('throwKVRequiredError', () => {
	it('should throw an error with the service name', () => {
		expect(() =>
			throwKVRequiredError({
				serviceName: 'fluxer_api',
				configPath: 'internal.kv.url',
			}),
		).toThrow('fluxer_api requires KV-backed rate limiting');
	});
	it('should include the config path in the error message', () => {
		expect(() =>
			throwKVRequiredError({
				serviceName: 'TestService',
				configPath: 'config.kv.connection_string',
			}),
		).toThrow('config.kv.connection_string is not set');
	});
	it('should construct complete error message with all parts', () => {
		let errorMessage = '';
		try {
			throwKVRequiredError({
				serviceName: 'fluxer_admin',
				configPath: 'admin.kv.endpoint',
			});
		} catch (error) {
			if (error instanceof Error) {
				errorMessage = error.message;
			}
		}
		expect(errorMessage).toContain('fluxer_admin requires KV-backed rate limiting');
		expect(errorMessage).toContain('admin.kv.endpoint is not set');
		expect(errorMessage).toContain('internal.kv must be configured for distributed rate limiting');
	});
	it('should always throw (never return)', () => {
		const fn = () =>
			throwKVRequiredError({
				serviceName: 'test',
				configPath: 'test.path',
			});
		expect(fn).toThrow(Error);
	});
	it('should handle empty service name', () => {
		expect(() =>
			throwKVRequiredError({
				serviceName: '',
				configPath: 'internal.kv',
			}),
		).toThrow('requires KV-backed rate limiting');
	});
	it('should handle empty config path', () => {
		expect(() =>
			throwKVRequiredError({
				serviceName: 'TestService',
				configPath: '',
			}),
		).toThrow('is not set');
	});
});
