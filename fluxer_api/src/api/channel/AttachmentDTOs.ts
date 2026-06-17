// SPDX-License-Identifier: AGPL-3.0-or-later

import type {
	ClientAttachmentReferenceRequest,
	ClientAttachmentRequest,
} from '@fluxer/schema/src/domains/message/AttachmentSchemas';

export interface UploadedAttachment {
	id: number;
	filename: string;
	upload_filename: string;
	file_size: number;
	content_type: string;
}

export interface AttachmentToProcess {
	id: number;
	filename: string;
	upload_filename: string;
	title: string | null;
	description: string | null;
	flags: number;
	file_size: number;
	content_type: string;
	duration?: number | null;
	waveform?: string | null;
}

export function mergeUploadWithClientData(
	uploaded: UploadedAttachment,
	clientData?: ClientAttachmentRequest | ClientAttachmentReferenceRequest,
): AttachmentToProcess {
	return {
		id: uploaded.id,
		filename: uploaded.filename,
		upload_filename: uploaded.upload_filename,
		file_size: uploaded.file_size,
		content_type: uploaded.content_type,
		duration: ('duration' in (clientData ?? {}) ? (clientData as ClientAttachmentRequest).duration : null) ?? null,
		waveform: ('waveform' in (clientData ?? {}) ? (clientData as ClientAttachmentRequest).waveform : null) ?? null,
		title: clientData?.title ?? null,
		description: clientData?.description ?? null,
		flags: 'flags' in (clientData ?? {}) ? (clientData as ClientAttachmentRequest).flags : 0,
	};
}

export type AttachmentRequestData = AttachmentToProcess | ClientAttachmentRequest | ClientAttachmentReferenceRequest;
