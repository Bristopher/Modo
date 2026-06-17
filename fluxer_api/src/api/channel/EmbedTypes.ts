// SPDX-License-Identifier: AGPL-3.0-or-later

import type {RichEmbedMediaRequestShape} from '@fluxer/schema/src/domains/message/MessageRequestSchemas';
import type {z} from 'zod';

export interface RichEmbedMediaWithMetadata extends z.infer<typeof RichEmbedMediaRequestShape> {
	_attachmentMetadata?: {
		width: number | null;
		height: number | null;
		content_type: string;
		content_hash: string | null;
		placeholder: string | null;
		flags: number;
		duration: number | null;
		nsfw: boolean | null;
	};
}
