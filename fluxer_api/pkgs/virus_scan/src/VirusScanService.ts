// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';
import {createReadStream} from 'node:fs';
import path from 'node:path';
import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {IVirusHashCache} from '@pkgs/virus_scan/src/cache/IVirusHashCache';
import type {IVirusScanFailureReporter} from '@pkgs/virus_scan/src/failures/IVirusScanFailureReporter';
import {NoopVirusScanFailureReporter} from '@pkgs/virus_scan/src/failures/NoopVirusScanFailureReporter';
import type {IVirusScanProvider} from '@pkgs/virus_scan/src/IVirusScanProvider';
import type {IVirusScanService} from '@pkgs/virus_scan/src/IVirusScanService';
import type {VirusScanResult} from '@pkgs/virus_scan/src/VirusScanResult';

interface VirusScanConfig {
	failOpen: boolean;
	cachedThreatLabel?: string;
}

interface VirusScanServiceDependencies {
	provider: IVirusScanProvider;
	virusHashCache: IVirusHashCache;
	logger: LoggerInterface;
	config: VirusScanConfig;
	failureReporter?: IVirusScanFailureReporter;
}

export class VirusScanService implements IVirusScanService {
	readonly enabled = true;
	private readonly cachedThreatLabel: string;
	private readonly failureReporter: IVirusScanFailureReporter;

	constructor(private dependencies: VirusScanServiceDependencies) {
		this.cachedThreatLabel = dependencies.config.cachedThreatLabel ?? 'Cached virus signature';
		this.failureReporter = dependencies.failureReporter ?? new NoopVirusScanFailureReporter();
	}

	async initialize(): Promise<void> {
		await this.failureReporter.initialize();
	}

	async scanFile(filePath: string): Promise<VirusScanResult> {
		const filename = path.basename(filePath);
		const fileHash = await this.hashFile(filePath);
		const isCachedVirus = await this.dependencies.virusHashCache.isKnownVirusHash(fileHash);
		if (isCachedVirus) {
			return {
				isClean: false,
				threat: this.cachedThreatLabel,
				fileHash,
			};
		}
		try {
			const scanResult = await this.dependencies.provider.scanFile(filePath);
			if (scanResult.isClean) {
				return {
					isClean: true,
					fileHash,
				};
			}
			if (!scanResult.threat) {
				throw new Error('Virus scan provider returned infected status without threat name');
			}
			await this.dependencies.virusHashCache.cacheVirusHash(fileHash);
			return {
				isClean: false,
				threat: scanResult.threat,
				fileHash,
			};
		} catch (error) {
			this.dependencies.logger.error(
				{
					error: this.describeError(error),
					filename,
					fileHash,
				},
				'Virus scan failed',
			);
			await this.reportScanFailure(error, filename, fileHash);
			if (this.dependencies.config.failOpen) {
				return {
					isClean: true,
					fileHash,
				};
			}
			throw new Error(`Virus scan failed: ${this.describeError(error)}`);
		}
	}

	async scanBuffer(buffer: Buffer, filename: string): Promise<VirusScanResult> {
		const fileHash = this.createFileHashFromBuffer(buffer);
		const isCachedVirus = await this.dependencies.virusHashCache.isKnownVirusHash(fileHash);
		if (isCachedVirus) {
			return {
				isClean: false,
				threat: this.cachedThreatLabel,
				fileHash,
			};
		}
		try {
			const scanResult = await this.dependencies.provider.scanBuffer(buffer);
			if (scanResult.isClean) {
				return {
					isClean: true,
					fileHash,
				};
			}
			if (!scanResult.threat) {
				throw new Error('Virus scan provider returned infected status without threat name');
			}
			await this.dependencies.virusHashCache.cacheVirusHash(fileHash);
			return {
				isClean: false,
				threat: scanResult.threat,
				fileHash,
			};
		} catch (error) {
			this.dependencies.logger.error(
				{
					error: this.describeError(error),
					filename,
					fileHash,
				},
				'Virus scan failed',
			);
			await this.reportScanFailure(error, filename, fileHash);
			if (this.dependencies.config.failOpen) {
				return {
					isClean: true,
					fileHash,
				};
			}
			throw new Error(`Virus scan failed: ${this.describeError(error)}`);
		}
	}

	private createFileHashFromBuffer(buffer: Buffer): string {
		return crypto.createHash('sha256').update(buffer).digest('hex');
	}

	private async hashFile(filePath: string): Promise<string> {
		const hash = crypto.createHash('sha256');
		const stream = createReadStream(filePath, {highWaterMark: 1024 * 1024});
		try {
			for await (const chunk of stream) {
				hash.update(chunk as Buffer);
			}
		} catch (error) {
			stream.destroy();
			throw error;
		}
		return hash.digest('hex');
	}

	private describeError(error: unknown): string {
		if (typeof error === 'string') {
			return error;
		}
		if (error instanceof Error) {
			return error.message;
		}
		return 'Unknown error';
	}

	private async reportScanFailure(error: unknown, filename: string, fileHash: string): Promise<void> {
		try {
			await this.failureReporter.reportFailure({
				error,
				filename,
				fileHash,
				failOpen: this.dependencies.config.failOpen,
			});
		} catch (reportError) {
			this.dependencies.logger.warn(
				{
					error: this.describeError(reportError),
					filename,
					fileHash,
				},
				'Failed to report virus scan failure',
			);
		}
	}
}
