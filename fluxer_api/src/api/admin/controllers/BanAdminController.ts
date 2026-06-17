// SPDX-License-Identifier: AGPL-3.0-or-later

import {AdminACLs} from '@fluxer/constants/src/AdminACLs';
import {
	BanAvatarHashRequest,
	BanCheckResponseSchema,
	BanEmailRequest,
	BanFileShaRequest,
	BanIpRequest,
	BanPhraseRequest,
	BanProfileSubstringRequest,
	BanUrlDomainRequest,
	BanUrlRequest,
	BanUserAvatarRequest,
	BanUserAvatarResponseSchema,
	BulkBanFileShasRequest,
	BulkJobResponse,
	CheckAvatarHashRequest,
	CheckFileShaRequest,
	CheckUrlBlocklistRequest,
	SuspiciousEmailDomainRequest,
	UnbanFileShaRequest,
	UnbanUrlDomainRequest,
	UnbanUrlRequest,
} from '@fluxer/schema/src/domains/admin/AdminSchemas';
import {requireAdminACL} from '../../middleware/AdminMiddleware';
import {RateLimitMiddleware} from '../../middleware/RateLimitMiddleware';
import {OpenAPI} from '../../middleware/ResponseTypeMiddleware';
import {getWorkerService} from '../../middleware/ServiceRegistry';
import {RateLimitConfigs} from '../../RateLimitConfig';
import type {HonoApp} from '../../types/HonoEnv';
import {Validator} from '../../Validator';

export function BanAdminController(app: HonoApp) {
	app.post(
		'/admin/bans/ip/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_IP_ADD),
		Validator('json', BanIpRequest),
		OpenAPI({
			operationId: 'add_ip_ban',
			summary: 'Add IP ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban one or more IP addresses from accessing the platform. Users connecting from banned IPs will be denied service. Can be applied retroactively.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banIp(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/ip/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_IP_REMOVE),
		Validator('json', BanIpRequest),
		OpenAPI({
			operationId: 'remove_ip_ban',
			summary: 'Remove IP ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Lift a previously applied IP ban, allowing traffic from those addresses again. Used for appeals or when bans were applied in error.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanIp(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/ip/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_IP_CHECK),
		Validator('json', BanIpRequest),
		OpenAPI({
			operationId: 'check_ip_ban_status',
			summary: 'Check IP ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Query whether one or more IP addresses are currently banned. Returns the ban status and any associated metadata like reason or expiration.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkIpBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/email/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_EMAIL_ADD),
		Validator('json', BanEmailRequest),
		OpenAPI({
			operationId: 'add_email_ban',
			summary: 'Add email ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban one or more email addresses from registering or creating accounts. Users attempting to use banned emails will be blocked.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banEmail(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/email/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_EMAIL_REMOVE),
		Validator('json', BanEmailRequest),
		OpenAPI({
			operationId: 'remove_email_ban',
			summary: 'Remove email ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Lift a previously applied email ban, allowing the address to be used for new registrations. Used for appeals or error correction.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanEmail(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/email/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_EMAIL_CHECK),
		Validator('json', BanEmailRequest),
		OpenAPI({
			operationId: 'check_email_ban_status',
			summary: 'Check email ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Query whether one or more email addresses are currently banned from registration. Returns the ban status and metadata.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkEmailBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/suspicious-email-domains/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.SUSPICIOUS_EMAIL_DOMAIN_ADD),
		Validator('json', SuspiciousEmailDomainRequest),
		OpenAPI({
			operationId: 'add_suspicious_email_domain',
			summary: 'Add suspicious email domain',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Flag an email domain as suspicious. Registration is not blocked, but new accounts using this domain are required to verify a phone number before they can act on the platform. The list itself is not exposed to users — they only see the verified-phone gate.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.addSuspiciousEmailDomain(
				ctx.req.valid('json'),
				adminUserId,
				auditLogReason,
			);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/suspicious-email-domains/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.SUSPICIOUS_EMAIL_DOMAIN_REMOVE),
		Validator('json', SuspiciousEmailDomainRequest),
		OpenAPI({
			operationId: 'remove_suspicious_email_domain',
			summary: 'Remove suspicious email domain flag',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Remove a domain from the suspicious list. New registrations from this domain will no longer be auto-required to verify a phone number on signup.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.removeSuspiciousEmailDomain(
				ctx.req.valid('json'),
				adminUserId,
				auditLogReason,
			);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/suspicious-email-domains/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.SUSPICIOUS_EMAIL_DOMAIN_CHECK),
		Validator('json', SuspiciousEmailDomainRequest),
		OpenAPI({
			operationId: 'check_suspicious_email_domain',
			summary: 'Check suspicious email domain status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether an email domain is currently flagged as suspicious.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkSuspiciousEmailDomain(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/phrase/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PHRASE_ADD),
		Validator('json', BanPhraseRequest),
		OpenAPI({
			operationId: 'add_phrase_ban',
			summary: 'Add phrase ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban a phrase. Matching is case-insensitive and also normalizes common bypass tricks such as inserted whitespace, punctuation, invisible characters, and compatibility glyphs.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banPhrase(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/phrase/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PHRASE_REMOVE),
		Validator('json', BanPhraseRequest),
		OpenAPI({
			operationId: 'remove_phrase_ban',
			summary: 'Remove phrase ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied phrase ban, allowing messages containing that phrase again.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanPhrase(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/phrase/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PHRASE_CHECK),
		Validator('json', BanPhraseRequest),
		OpenAPI({
			operationId: 'check_phrase_ban_status',
			summary: 'Check phrase ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether a phrase is currently banned.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkPhraseBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/url/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_ADD),
		Validator('json', BanUrlRequest),
		OpenAPI({
			operationId: 'add_url_ban',
			summary: 'Add URL ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban one or more URLs from being posted on the platform. Messages containing banned URLs will be blocked.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banUrl(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/url/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_REMOVE),
		Validator('json', UnbanUrlRequest),
		OpenAPI({
			operationId: 'remove_url_ban',
			summary: 'Remove URL ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied URL ban, allowing the URL to be posted again.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanUrl(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/url/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_CHECK),
		Validator('json', CheckUrlBlocklistRequest),
		OpenAPI({
			operationId: 'check_url_ban_status',
			summary: 'Check URL ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether one or more URLs are currently banned from being posted.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkUrlBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/url-domain/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_DOMAIN_ADD),
		Validator('json', BanUrlDomainRequest),
		OpenAPI({
			operationId: 'add_url_domain_ban',
			summary: 'Add URL domain ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban an entire URL domain from being linked on the platform. All URLs under the banned domain will be blocked.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banUrlDomain(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/url-domain/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_DOMAIN_REMOVE),
		Validator('json', UnbanUrlDomainRequest),
		OpenAPI({
			operationId: 'remove_url_domain_ban',
			summary: 'Remove URL domain ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied URL domain ban, allowing links to that domain again.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanUrlDomain(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/url-domain/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_URL_DOMAIN_CHECK),
		Validator('json', BanUrlDomainRequest),
		OpenAPI({
			operationId: 'check_url_domain_ban_status',
			summary: 'Check URL domain ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether a URL domain is currently banned from being linked.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkUrlDomainBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/file-sha/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_FILE_SHA_ADD),
		Validator('json', BanFileShaRequest),
		OpenAPI({
			operationId: 'add_file_sha_ban',
			summary: 'Add file SHA ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Ban one or more files by SHA hash. Uploads matching the banned hash will be rejected.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banFileSha(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/file-sha/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_FILE_SHA_REMOVE),
		Validator('json', UnbanFileShaRequest),
		OpenAPI({
			operationId: 'remove_file_sha_ban',
			summary: 'Remove file SHA ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied file SHA ban, allowing uploads of that file again.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanFileSha(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/file-sha/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_FILE_SHA_CHECK),
		Validator('json', CheckFileShaRequest),
		OpenAPI({
			operationId: 'check_file_sha_ban_status',
			summary: 'Check file SHA ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether one or more file SHA hashes are currently banned.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkFileShaBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/avatar-hash/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_AVATAR_HASH_ADD),
		Validator('json', BanAvatarHashRequest),
		OpenAPI({
			operationId: 'add_avatar_hash_ban',
			summary: 'Add avatar hash ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban one or more 8-char MD5-prefix avatar hashes. Avatars matching the banned hash will be rejected on upload.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banAvatarHash(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/avatar-hash/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_AVATAR_HASH_REMOVE),
		Validator('json', CheckAvatarHashRequest),
		OpenAPI({
			operationId: 'remove_avatar_hash_ban',
			summary: 'Remove avatar hash ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied avatar-hash ban.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanAvatarHash(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/avatar-hash/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_AVATAR_HASH_CHECK),
		Validator('json', CheckAvatarHashRequest),
		OpenAPI({
			operationId: 'check_avatar_hash_ban_status',
			summary: 'Check avatar hash ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether any of the provided avatar hashes are banned.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkAvatarHashBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/users/:user_id/ban-avatar',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_AVATAR_HASH_ADD),
		Validator('json', BanUserAvatarRequest),
		OpenAPI({
			operationId: 'ban_user_avatar',
			summary: "Ban this user's current avatar",
			responseSchema: BanUserAvatarResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				"Reads the user's current avatar_hash, strips any animation prefix, and adds the 8-char hash to the avatar blocklist. Returns the banned hash.",
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const userId = ctx.req.param('user_id');
			const body = ctx.req.valid('json');
			return ctx.json(
				await adminService.banManagementService.banUserAvatar({user_id: userId, ...body}, adminUserId, auditLogReason),
			);
		},
	);
	app.post(
		'/admin/bans/profile-substring/add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PROFILE_SUBSTRING_ADD),
		Validator('json', BanProfileSubstringRequest),
		OpenAPI({
			operationId: 'add_profile_substring_ban',
			summary: 'Add profile-substring ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Ban a substring within a specific profile field (username, global_name, nickname, bio, or pronouns). Matching reuses the phrase blocklist normalization (whitespace, punctuation, zero-width, lookalikes).',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.banProfileSubstring(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/profile-substring/remove',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PROFILE_SUBSTRING_REMOVE),
		Validator('json', BanProfileSubstringRequest),
		OpenAPI({
			operationId: 'remove_profile_substring_ban',
			summary: 'Remove profile-substring ban',
			responseSchema: null,
			statusCode: 204,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Lift a previously applied profile-substring ban.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			await adminService.banManagementService.unbanProfileSubstring(ctx.req.valid('json'), adminUserId, auditLogReason);
			return ctx.body(null, 204);
		},
	);
	app.post(
		'/admin/bans/profile-substring/check',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_PROFILE_SUBSTRING_CHECK),
		Validator('json', BanProfileSubstringRequest),
		OpenAPI({
			operationId: 'check_profile_substring_ban_status',
			summary: 'Check profile-substring ban status',
			responseSchema: BanCheckResponseSchema,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description: 'Query whether any of the provided substrings are banned for the given scope.',
		}),
		async (ctx) => {
			const adminService = ctx.get('adminService');
			return ctx.json(await adminService.banManagementService.checkProfileSubstringBan(ctx.req.valid('json')));
		},
	);
	app.post(
		'/admin/bans/file-sha/bulk-add',
		RateLimitMiddleware(RateLimitConfigs.ADMIN_BAN_OPERATION),
		requireAdminACL(AdminACLs.BAN_FILE_SHA_ADD),
		Validator('json', BulkBanFileShasRequest),
		OpenAPI({
			operationId: 'bulk_ban_file_shas',
			summary: 'Bulk-ban file SHAs as a background job',
			responseSchema: BulkJobResponse,
			statusCode: 200,
			security: ['adminApiKey'],
			tags: ['Admin'],
			description:
				'Enqueue a background job that bans many file SHAs at once. Returns a job_id immediately; observe progress at /admin/jobs/:job_id.',
		}),
		async (ctx) => {
			const adminUserId = ctx.get('adminUserId');
			const auditLogReason = ctx.get('auditLogReason');
			const body = ctx.req.valid('json');
			const workerService = getWorkerService();
			const jobId = await workerService.addJob(
				'bulkBanFileShas',
				{
					sha256_list: body.sha256_list,
					admin_user_id: adminUserId.toString(),
					audit_log_reason: auditLogReason,
				},
				{requestedByUserId: adminUserId, ...(auditLogReason && {auditLogReason})},
			);
			return ctx.json({job_id: jobId.toString()});
		},
	);
}
