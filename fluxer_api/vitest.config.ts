// SPDX-License-Identifier: AGPL-3.0-or-later

import {availableParallelism} from 'node:os';
import tsconfigPaths from 'vite-tsconfig-paths';
import {configDefaults, defineConfig} from 'vitest/config';

function parseParallelInteger(value: string | undefined, fallback: number): number {
	if (!value) {
		return fallback;
	}
	const parsed = Number.parseInt(value, 10);
	if (!Number.isInteger(parsed) || parsed < 1) {
		return fallback;
	}
	return parsed;
}

function resolveDefaultParallelWorkers(): number {
	const halfCoreCountMinusOne = Math.floor(availableParallelism() / 2) - 1;
	return Math.max(2, halfCoreCountMinusOne);
}

const DEFAULT_PARALLEL_WORKERS = resolveDefaultParallelWorkers();
const configuredMaxWorkers = parseParallelInteger(process.env.API_TEST_MAX_WORKERS, DEFAULT_PARALLEL_WORKERS);
const configuredMaxConcurrency = parseParallelInteger(process.env.API_TEST_MAX_CONCURRENCY, configuredMaxWorkers);

export default defineConfig({
	root: process.cwd(),
	plugins: [tsconfigPaths()],
	cacheDir: './node_modules/.vitest',
	test: {
		globals: true,
		environment: 'node',
		setupFiles: ['./src/api/test/Setup.ts'],
		include: ['src/**/*.{test,spec}.{ts,tsx}'],
		exclude: [
			...configDefaults.exclude,
			'pkgs/**',
			'../fluxer_desktop/**',
			'**/target/**',
			'**/*Integration.test.ts',
			'**/*ExttestIntegration.test.ts',
		],
		pool: 'threads',
		fileParallelism: true,
		maxWorkers: configuredMaxWorkers,
		maxConcurrency: configuredMaxConcurrency,
		isolate: true,
		testTimeout: 40000,
		hookTimeout: 20000,
		reporters: ['default', 'json'],
		outputFile: './test-results.json',
		coverage: {
			provider: 'v8',
			reporter: ['text', 'text-summary', 'json', 'html'],
			reportsDirectory: './coverage',
			exclude: [
				'**/node_modules/tests/test*.test.tsx',
				'**/*.test.ts',
				'**/Setup.tsx',
				'**/TestConstants.tsx',
				'**/TestRequestBuilder.tsx',
				'**/TestHelpers.tsx',
			],
		},
	},
});
