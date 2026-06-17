// SPDX-License-Identifier: AGPL-3.0-or-later

import {APIErrorCodes} from '@fluxer/constants/src/ApiErrorCodes';
import {
	SMS_TWILIO_DEFAULT_LOOKUP_API_URL,
	SMS_TWILIO_DEFAULT_VERIFY_API_URL,
} from '@fluxer/constants/src/SmsVerificationConstants';
import {InvalidPhoneNumberError} from '@fluxer/errors/src/domains/auth/InvalidPhoneNumberError';
import {SmsVerificationUnavailableError} from '@fluxer/errors/src/domains/auth/SmsVerificationUnavailableError';
import {RateLimitError} from '@fluxer/errors/src/domains/core/RateLimitError';
import {createLogger} from '@fluxer/logger/src/Logger';
import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {PhoneLineType, PhoneLookupResult} from '@pkgs/sms/src/PhoneLookupTypes';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import type {SmsVerificationStartOptions, SmsVerificationStartResult} from '@pkgs/sms/src/SmsVerificationTypes';
import {maskPhoneNumber} from '@pkgs/sms/src/SmsVerificationUtils';

const TWILIO_INVALID_PHONE_ERROR_CODE = 21211;
const TWILIO_TOO_MANY_REQUESTS_ERROR_CODE = 20429;
const TWILIO_MAX_SEND_ATTEMPTS_ERROR_CODE = 60203;
const TWILIO_MAX_CHECK_ATTEMPTS_ERROR_CODE = 60202;
const TWILIO_CONCURRENT_REQUESTS_ERROR_CODE = 60212;
const TWILIO_FRAUD_GUARD_BLOCK_ERROR_CODE = 60410;
const TWILIO_FRAUD_PREVENTION_BLOCK_ERROR_CODE = 60412;
const TWILIO_DEFAULT_BUSY_RETRY_AFTER_SECONDS = 60;
const TWILIO_VERIFY_WINDOW_RETRY_AFTER_SECONDS = 10 * 60;
const TWILIO_FRAUD_BLOCK_RETRY_AFTER_SECONDS = 12 * 60 * 60;

interface TwilioErrorResponse {
	code?: number;
	message?: string;
}

interface TwilioVerificationCheckResponse {
	status?: string;
}

interface TwilioVerificationStartResponse {
	channel?: string;
}

type TwilioCooldownScope = 'account' | 'phone' | 'account_and_phone';

interface StartVerificationSentryContext extends Record<string, unknown> {
	smsProvider: 'twilio';
	smsOperation: 'start_verification';
	twilioEndpoint: 'Verifications';
	phone: string;
	twilioStatus?: number;
	twilioCode?: number;
	twilioMessage?: string;
	twilioRequestError?: string;
}

interface TwilioLookupV2Response {
	valid?: boolean;
	country_code?: string;
	line_type_intelligence?: {
		type?: string;
		carrier_name?: string;
		error_code?: number;
	};
	sms_pumping_risk?: {
		sms_pumping_risk_score?: number;
		error_code?: number;
	};
}

export interface TwilioSmsProviderConfig {
	accountSid: string;
	authToken: string;
	verifyServiceSid: string;
	verifyApiUrl?: string;
	lookupApiUrl?: string;
	lookupTimeoutMs?: number;
}

interface TwilioSmsProviderDependencies {
	config: TwilioSmsProviderConfig;
	logger?: LoggerInterface;
	fetchFn?: typeof fetch;
}

const DEFAULT_LOOKUP_TIMEOUT_MS = 3000;
const KNOWN_LINE_TYPES: ReadonlySet<PhoneLineType> = new Set<PhoneLineType>([
	'mobile',
	'landline',
	'fixedVoip',
	'nonFixedVoip',
	'personal',
	'tollFree',
	'premium',
	'sharedCost',
	'uan',
	'voicemail',
	'pager',
	'unknown',
]);

function normalizeLineType(raw: string | undefined): PhoneLineType | null {
	if (!raw) return null;
	return KNOWN_LINE_TYPES.has(raw as PhoneLineType) ? (raw as PhoneLineType) : 'unknown';
}

export class SmsVerificationStartError extends Error {
	readonly sentryContext: StartVerificationSentryContext;

	constructor(sentryContext: StartVerificationSentryContext, options?: ErrorOptions) {
		super('Failed to start SMS verification', options);
		this.name = 'SmsVerificationStartError';
		this.sentryContext = sentryContext;
	}
}

export class TwilioVerificationRateLimitError extends RateLimitError {
	readonly twilioCode?: number;
	readonly twilioStatus?: number;
	readonly cooldownScope: TwilioCooldownScope;
	readonly cooldownMs: number;

	constructor(args: {
		message: string;
		retryAfterSeconds: number;
		twilioCode?: number;
		twilioStatus?: number;
		cooldownScope: TwilioCooldownScope;
		cooldownMs?: number;
		scope?: 'shared' | 'user';
	}) {
		const retryAfterSeconds = Math.max(1, Math.ceil(args.retryAfterSeconds));
		super({
			code: APIErrorCodes.PHONE_RATE_LIMIT_EXCEEDED,
			message: args.message,
			retryAfter: retryAfterSeconds,
			retryAfterDecimal: retryAfterSeconds,
			limit: 1,
			resetTime: new Date(Date.now() + retryAfterSeconds * 1000),
			resetAfterDecimal: retryAfterSeconds,
			scope: args.scope ?? 'shared',
		});
		this.name = 'TwilioVerificationRateLimitError';
		this.twilioCode = args.twilioCode;
		this.twilioStatus = args.twilioStatus;
		this.cooldownScope = args.cooldownScope;
		this.cooldownMs = args.cooldownMs ?? retryAfterSeconds * 1000;
	}
}

export class TwilioSmsProvider implements ISmsProvider {
	private readonly verifyApiUrl: string;
	private readonly lookupApiUrl: string;
	private readonly lookupTimeoutMs: number;
	private readonly logger: LoggerInterface;
	private readonly config: TwilioSmsProviderConfig;
	private readonly fetchFn: typeof fetch;

	constructor({config, logger, fetchFn = fetch}: TwilioSmsProviderDependencies) {
		this.verifyApiUrl = config.verifyApiUrl ?? SMS_TWILIO_DEFAULT_VERIFY_API_URL;
		this.lookupApiUrl = config.lookupApiUrl ?? SMS_TWILIO_DEFAULT_LOOKUP_API_URL;
		this.lookupTimeoutMs = config.lookupTimeoutMs ?? DEFAULT_LOOKUP_TIMEOUT_MS;
		this.logger = logger ?? createLogger('@pkgs/sms/src');
		this.config = config;
		this.fetchFn = fetchFn;
	}

	async startVerification(phone: string): Promise<void> {
		await this.startVerificationWithResult(phone);
	}

	async startVerificationWithResult(
		phone: string,
		options: SmsVerificationStartOptions = {},
	): Promise<SmsVerificationStartResult> {
		const requestedChannel = options.channel ?? 'sms';
		const requestBody: Record<string, string> = {
			To: phone,
			Channel: requestedChannel,
		};
		if (options.deviceIp) {
			requestBody.DeviceIp = options.deviceIp;
		}
		if (options.rateLimits) {
			for (const [key, value] of Object.entries(options.rateLimits)) {
				if (!key || !value) continue;
				requestBody[`RateLimits[${key}]`] = value;
			}
		}
		let response: Response;
		try {
			response = await this.requestTwilio('Verifications', requestBody);
		} catch (error) {
			const sentryContext = this.createStartVerificationSentryContext(phone, {
				requestError: error,
			});
			this.logger.error(sentryContext, '[TwilioSmsProvider] Twilio request failed while starting SMS verification');
			throw new SmsVerificationUnavailableError();
		}
		if (response.ok) {
			let parsed: TwilioVerificationStartResponse | null = null;
			try {
				parsed = (await response.json()) as TwilioVerificationStartResponse;
			} catch (error) {
				this.logger.warn(
					{
						error: error instanceof Error ? error.message : String(error),
						phone: maskPhoneNumber(phone),
						channel: requestedChannel,
					},
					'[TwilioSmsProvider] Verification start response JSON parse failed',
				);
			}
			return {
				channel: parsed?.channel ?? requestedChannel,
			};
		}
		const body = await this.parseErrorBody(response);
		if (body?.code === TWILIO_INVALID_PHONE_ERROR_CODE) {
			throw new InvalidPhoneNumberError();
		}
		const rateLimitError = this.createRateLimitError(response, body, 'Verifications');
		if (rateLimitError) {
			throw rateLimitError;
		}
		const sentryContext = this.createStartVerificationSentryContext(phone, {
			response,
			body,
		});
		this.logger.error(sentryContext, '[TwilioSmsProvider] Failed to start SMS verification');
		throw new SmsVerificationUnavailableError();
	}

	async checkVerification(phone: string, code: string): Promise<boolean> {
		let response: Response;
		try {
			response = await this.requestTwilio('VerificationCheck', {
				To: phone,
				Code: code,
			});
		} catch (error) {
			this.logger.error(
				{error: error instanceof Error ? error.message : String(error), phone: maskPhoneNumber(phone)},
				'[TwilioSmsProvider] Twilio request failed while checking SMS verification',
			);
			throw new SmsVerificationUnavailableError();
		}
		if (!response.ok) {
			const body = await this.parseErrorBody(response);
			const rateLimitError = this.createRateLimitError(response, body, 'VerificationCheck');
			if (rateLimitError) {
				throw rateLimitError;
			}
			if (response.status >= 500) {
				this.logger.error(
					{
						status: response.status,
						code: body?.code,
						message: body?.message,
						phone: maskPhoneNumber(phone),
					},
					'[TwilioSmsProvider] Verification check failed with provider error',
				);
				throw new SmsVerificationUnavailableError();
			}
			return false;
		}
		const body = (await response.json()) as TwilioVerificationCheckResponse;
		return body.status === 'approved';
	}

	async lookupPhone(phone: string): Promise<PhoneLookupResult | null> {
		const url = `${this.lookupApiUrl}/PhoneNumbers/${encodeURIComponent(phone)}?Fields=line_type_intelligence,sms_pumping_risk`;
		const auth = Buffer.from(`${this.config.accountSid}:${this.config.authToken}`).toString('base64');
		const abort = new AbortController();
		const timer = setTimeout(() => abort.abort(), this.lookupTimeoutMs);
		let response: Response;
		try {
			response = await this.fetchFn(url, {
				method: 'GET',
				headers: {Authorization: `Basic ${auth}`},
				signal: abort.signal,
			});
		} catch (error) {
			this.logger.warn(
				{error: error instanceof Error ? error.message : String(error), phone: maskPhoneNumber(phone)},
				'[TwilioSmsProvider] Lookup request failed (fail-open)',
			);
			return null;
		} finally {
			clearTimeout(timer);
		}
		if (response.status === 404) {
			return {
				valid: false,
				lineType: null,
				countryCode: null,
				carrierName: null,
				smsPumpingRiskScore: null,
			};
		}
		if (!response.ok) {
			const body = await this.parseErrorBody(response);
			this.logger.warn(
				{
					status: response.status,
					code: body?.code,
					message: body?.message,
					phone: maskPhoneNumber(phone),
				},
				'[TwilioSmsProvider] Lookup returned non-OK (fail-open)',
			);
			return null;
		}
		let parsed: TwilioLookupV2Response;
		try {
			parsed = (await response.json()) as TwilioLookupV2Response;
		} catch (error) {
			this.logger.warn(
				{error: error instanceof Error ? error.message : String(error), phone: maskPhoneNumber(phone)},
				'[TwilioSmsProvider] Lookup response JSON parse failed (fail-open)',
			);
			return null;
		}
		const countryCode = parsed.country_code ?? null;
		const carrierName = parsed.line_type_intelligence?.carrier_name ?? null;
		const ltiErrorCode = parsed.line_type_intelligence?.error_code ?? null;
		const sprErrorCode = parsed.sms_pumping_risk?.error_code ?? null;
		if (ltiErrorCode != null) {
			this.logger.warn(
				{phone: maskPhoneNumber(phone), countryCode, ltiErrorCode},
				'[TwilioSmsProvider] Lookup line_type_intelligence reported error_code',
			);
		}
		if (sprErrorCode != null) {
			this.logger.warn(
				{phone: maskPhoneNumber(phone), countryCode, sprErrorCode},
				'[TwilioSmsProvider] Lookup sms_pumping_risk reported error_code',
			);
		}
		const rawScore = parsed.sms_pumping_risk?.sms_pumping_risk_score;
		const smsPumpingRiskScore = typeof rawScore === 'number' && Number.isFinite(rawScore) ? rawScore : null;
		return {
			valid: parsed.valid !== false,
			lineType: normalizeLineType(parsed.line_type_intelligence?.type),
			countryCode,
			carrierName,
			smsPumpingRiskScore,
		};
	}

	private async requestTwilio(
		endpoint: 'Verifications' | 'VerificationCheck',
		body: Record<string, string>,
	): Promise<Response> {
		const url = `${this.verifyApiUrl}/Services/${this.config.verifyServiceSid}/${endpoint}`;
		const auth = Buffer.from(`${this.config.accountSid}:${this.config.authToken}`).toString('base64');
		return this.fetchFn(url, {
			method: 'POST',
			headers: {
				Authorization: `Basic ${auth}`,
				'Content-Type': 'application/x-www-form-urlencoded',
			},
			body: new URLSearchParams(body).toString(),
		});
	}

	private async parseErrorBody(response: Response): Promise<TwilioErrorResponse | null> {
		try {
			return (await response.json()) as TwilioErrorResponse;
		} catch {
			return null;
		}
	}

	private createStartVerificationSentryContext(
		phone: string,
		params: {
			response?: Pick<Response, 'status'>;
			body?: TwilioErrorResponse | null;
			requestError?: unknown;
		},
	): StartVerificationSentryContext {
		const sentryContext: StartVerificationSentryContext = {
			smsProvider: 'twilio',
			smsOperation: 'start_verification',
			twilioEndpoint: 'Verifications',
			phone: maskPhoneNumber(phone),
		};
		if (params.response) {
			sentryContext.twilioStatus = params.response.status;
		}
		if (params.body?.code !== undefined) {
			sentryContext.twilioCode = params.body.code;
		}
		if (params.body?.message !== undefined) {
			sentryContext.twilioMessage = params.body.message;
		}
		if (params.requestError !== undefined) {
			sentryContext.twilioRequestError =
				params.requestError instanceof Error ? params.requestError.message : String(params.requestError);
		}
		return sentryContext;
	}

	private createRateLimitError(
		response: Pick<Response, 'status'>,
		body: TwilioErrorResponse | null,
		endpoint: 'Verifications' | 'VerificationCheck',
	): TwilioVerificationRateLimitError | null {
		const twilioCode = body?.code;
		const twilioStatus = response.status;
		if (twilioCode === TWILIO_MAX_SEND_ATTEMPTS_ERROR_CODE) {
			return new TwilioVerificationRateLimitError({
				message: 'Too many verification texts were sent recently. Try again later.',
				retryAfterSeconds: TWILIO_VERIFY_WINDOW_RETRY_AFTER_SECONDS,
				twilioCode,
				twilioStatus,
				cooldownScope: 'phone',
			});
		}
		if (twilioCode === TWILIO_CONCURRENT_REQUESTS_ERROR_CODE) {
			return new TwilioVerificationRateLimitError({
				message: 'Too many verification requests are already in flight for this number. Try again later.',
				retryAfterSeconds: TWILIO_VERIFY_WINDOW_RETRY_AFTER_SECONDS,
				twilioCode,
				twilioStatus,
				cooldownScope: 'phone',
			});
		}
		if (twilioCode === TWILIO_MAX_CHECK_ATTEMPTS_ERROR_CODE && endpoint === 'VerificationCheck') {
			return new TwilioVerificationRateLimitError({
				message: 'Too many verification code checks were attempted. Request a new code later.',
				retryAfterSeconds: TWILIO_VERIFY_WINDOW_RETRY_AFTER_SECONDS,
				twilioCode,
				twilioStatus,
				cooldownScope: 'phone',
			});
		}
		if (twilioCode === TWILIO_FRAUD_GUARD_BLOCK_ERROR_CODE || twilioCode === TWILIO_FRAUD_PREVENTION_BLOCK_ERROR_CODE) {
			return new TwilioVerificationRateLimitError({
				message: 'Phone verification is temporarily blocked for this destination. Try again later.',
				retryAfterSeconds: TWILIO_FRAUD_BLOCK_RETRY_AFTER_SECONDS,
				twilioCode,
				twilioStatus,
				cooldownScope: 'phone',
			});
		}
		if (twilioCode === TWILIO_TOO_MANY_REQUESTS_ERROR_CODE || (twilioCode == null && twilioStatus === 429)) {
			return new TwilioVerificationRateLimitError({
				message: 'Phone verification is temporarily busy. Try again shortly.',
				retryAfterSeconds: TWILIO_DEFAULT_BUSY_RETRY_AFTER_SECONDS,
				twilioCode,
				twilioStatus,
				cooldownScope: 'account',
				scope: 'user',
			});
		}
		return null;
	}
}
