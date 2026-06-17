// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IKVPipeline} from '@pkgs/kv_client/src/IKVProvider';
import {createStringEntriesFromPairs} from '@pkgs/kv_client/src/KVCommandArguments';
import type {ChainableCommander} from 'ioredis';

type PipelineExecResult = [Error | null, unknown];

interface KVPipelineOptions {
	createCommander: () => ChainableCommander;
	normalizeError: (command: string, error: unknown) => Error;
	mode: 'pipeline' | 'multi';
}

export class KVPipeline implements IKVPipeline {
	private readonly createCommander: () => ChainableCommander;
	private readonly normalizeError: (command: string, error: unknown) => Error;
	private readonly mode: 'pipeline' | 'multi';
	private commander: ChainableCommander;

	constructor(options: KVPipelineOptions) {
		this.createCommander = options.createCommander;
		this.normalizeError = options.normalizeError;
		this.mode = options.mode;
		this.commander = options.createCommander();
	}

	get(key: string): this {
		this.commander.get(key);
		return this;
	}

	set(key: string, value: string): this {
		this.commander.set(key, value);
		return this;
	}

	setex(key: string, ttlSeconds: number, value: string): this {
		this.commander.setex(key, ttlSeconds, value);
		return this;
	}

	del(key: string): this {
		this.commander.del(key);
		return this;
	}

	expire(key: string, ttlSeconds: number): this {
		this.commander.expire(key, ttlSeconds);
		return this;
	}

	sadd(key: string, ...members: Array<string>): this {
		this.commander.sadd(key, ...members);
		return this;
	}

	srem(key: string, ...members: Array<string>): this {
		this.commander.srem(key, ...members);
		return this;
	}

	zadd(key: string, score: number, value: string): this {
		this.commander.zadd(key, score, value);
		return this;
	}

	zrem(key: string, ...members: Array<string>): this {
		this.commander.zrem(key, ...members);
		return this;
	}

	hgetall(key: string): this {
		this.commander.hgetall(key);
		return this;
	}

	mset(...args: Array<string>): this {
		const entries = createStringEntriesFromPairs(args);
		if (entries.length === 0) {
			return this;
		}
		const pairs = entries.flatMap((entry) => [entry.key, entry.value]);
		this.commander.mset(...pairs);
		return this;
	}

	async exec(): Promise<Array<PipelineExecResult>> {
		const command = `${this.mode}.exec`;
		try {
			const rawResults = (await this.commander.exec()) as Array<PipelineExecResult> | null;
			this.commander = this.createCommander();
			if (rawResults === null) {
				return [];
			}
			return rawResults.map((result: PipelineExecResult) => {
				const [error, value] = result;
				return [error ? normalizePipelineError(error) : null, value] as PipelineExecResult;
			});
		} catch (error) {
			this.commander = this.createCommander();
			throw this.normalizeError(command, error);
		}
	}
}

function normalizePipelineError(error: unknown): Error {
	if (error instanceof Error) {
		return error;
	}
	return new Error(String(error));
}
