// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IStorageService} from './IStorageService';
import {StorageService} from './StorageService';

export function createStorageService(): IStorageService {
	return new StorageService();
}
