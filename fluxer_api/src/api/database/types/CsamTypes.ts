// SPDX-License-Identifier: AGPL-3.0-or-later

export interface NcmecAttachmentSubmissionRow {
	attachment_id: bigint;
	user_id: bigint | null;
	channel_id: bigint;
	message_id: bigint;
	filename: string;
	source_report_id: bigint | null;
	status: string;
	ncmec_report_id: string | null;
	ncmec_file_id: string | null;
	submitted_at: Date | null;
	submitted_by_admin_id: bigint | null;
	reporter_full_name: string | null;
	failure_reason: string | null;
	archive_completed_at: Date | null;
	content_deleted_at: Date | null;
	created_at: Date;
	updated_at: Date;
}

export const NCMEC_ATTACHMENT_SUBMISSION_COLUMNS = [
	'attachment_id',
	'user_id',
	'channel_id',
	'message_id',
	'filename',
	'source_report_id',
	'status',
	'ncmec_report_id',
	'ncmec_file_id',
	'submitted_at',
	'submitted_by_admin_id',
	'reporter_full_name',
	'failure_reason',
	'archive_completed_at',
	'content_deleted_at',
	'created_at',
	'updated_at',
] as const satisfies ReadonlyArray<keyof NcmecAttachmentSubmissionRow>;

export interface NcmecUserWorkflowRow {
	user_id: bigint;
	deleted_at: Date | null;
	deleted_by_admin_id: bigint | null;
	deletion_private_reason: string | null;
	deletion_ncmec_report_id: string | null;
	archive_id: bigint | null;
	archive_requested_at: Date | null;
	archive_requested_by_admin_id: bigint | null;
	archive_audit_log_reason: string | null;
	archive_completed_at: Date | null;
	deletion_job_queued_at: Date | null;
	previous_report_ids: Set<string> | null;
	created_at: Date;
	updated_at: Date;
}

export const NCMEC_USER_WORKFLOW_COLUMNS = [
	'user_id',
	'deleted_at',
	'deleted_by_admin_id',
	'deletion_private_reason',
	'deletion_ncmec_report_id',
	'archive_id',
	'archive_requested_at',
	'archive_requested_by_admin_id',
	'archive_audit_log_reason',
	'archive_completed_at',
	'deletion_job_queued_at',
	'previous_report_ids',
	'created_at',
	'updated_at',
] as const satisfies ReadonlyArray<keyof NcmecUserWorkflowRow>;
