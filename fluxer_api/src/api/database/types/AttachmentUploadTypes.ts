// SPDX-License-Identifier: AGPL-3.0-or-later

export interface AttachmentUploadTraceByKeyRow {
	upload_key: string;
	attachment_id: bigint | null;
	user_id: bigint;
	channel_id: bigint;
	filename: string;
	content_type: string;
	upload_mode: string;
	request_ip: string;
	requested_at: Date;
	completion_ip: string | null;
	completed_at: Date | null;
	created_at: Date;
	updated_at: Date;
}

export const ATTACHMENT_UPLOAD_TRACE_BY_KEY_COLUMNS = [
	'upload_key',
	'attachment_id',
	'user_id',
	'channel_id',
	'filename',
	'content_type',
	'upload_mode',
	'request_ip',
	'requested_at',
	'completion_ip',
	'completed_at',
	'created_at',
	'updated_at',
] as const satisfies ReadonlyArray<keyof AttachmentUploadTraceByKeyRow>;

export interface AttachmentUploadTraceByAttachmentRow {
	attachment_id: bigint;
	upload_key: string;
	user_id: bigint;
	channel_id: bigint;
	filename: string;
	content_type: string;
	upload_mode: string;
	request_ip: string;
	requested_at: Date;
	completion_ip: string | null;
	completed_at: Date | null;
	created_at: Date;
	updated_at: Date;
}

export const ATTACHMENT_UPLOAD_TRACE_BY_ATTACHMENT_COLUMNS = [
	'attachment_id',
	'upload_key',
	'user_id',
	'channel_id',
	'filename',
	'content_type',
	'upload_mode',
	'request_ip',
	'requested_at',
	'completion_ip',
	'completed_at',
	'created_at',
	'updated_at',
] as const satisfies ReadonlyArray<keyof AttachmentUploadTraceByAttachmentRow>;
