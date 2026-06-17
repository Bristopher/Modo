// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomBytes} from 'node:crypto';
import {FileSizeTooLargeError} from '@fluxer/errors/src/domains/core/FileSizeTooLargeError';
import {Config} from '../Config';
import type {IStorageService} from '../infrastructure/IStorageService';

const MAX_CSS_BYTES = 8 * 1024 * 1024;

export class ThemeService {
	constructor(private readonly storageService: IStorageService) {}

	async createTheme(css: string): Promise<{
		id: string;
	}> {
		const cssBytes = Buffer.from(css, 'utf-8');
		if (cssBytes.length > MAX_CSS_BYTES) {
			throw new FileSizeTooLargeError();
		}
		const themeId = randomBytes(8).toString('hex');
		await this.storageService.uploadObject({
			bucket: Config.s3.buckets.cdn,
			key: `themes/${themeId}.css`,
			body: cssBytes,
			contentType: 'text/css; charset=utf-8',
		});
		return {id: themeId};
	}
}
