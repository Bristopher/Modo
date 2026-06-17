// SPDX-License-Identifier: AGPL-3.0-or-later

import type {MessageEmbedFooter} from '../database/types/MessageTypes';
import {sanitizeOptionalAbsoluteUrlOrNull} from '../utils/UrlSanitizer';

export class EmbedFooter {
	readonly text: string | null;
	readonly iconUrl: string | null;

	constructor(footer: MessageEmbedFooter) {
		this.text = footer.text ?? null;
		this.iconUrl = sanitizeOptionalAbsoluteUrlOrNull(footer.icon_url);
	}

	toMessageEmbedFooter(): MessageEmbedFooter {
		return {
			text: this.text,
			icon_url: this.iconUrl,
		};
	}
}
