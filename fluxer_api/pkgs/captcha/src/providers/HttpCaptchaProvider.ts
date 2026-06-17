// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {CaptchaProviderType, ICaptchaProvider, VerifyCaptchaParams} from '@pkgs/captcha/src/ICaptchaProvider';
import {ms} from 'itty-time';

const DEFAULT_USER_AGENT = 'Mozilla/5.0 (compatible; Fluxerbot/1.0; +https://fluxer.app)';
const DEFAULT_TIMEOUT = ms('10 seconds');

export interface HttpCaptchaProviderOptions {
	secretKey: string;
	logger?: LoggerInterface;
	timeoutMs?: number;
	userAgent?: string;
	fetchFn?: typeof fetch;
}

interface CaptchaVerifyResponse {
	success: boolean;
	'error-codes'?: Array<string>;
	hostname?: string;
	challenge_ts?: string;
}

export abstract class HttpCaptchaProvider implements ICaptchaProvider {
	abstract readonly type: CaptchaProviderType;
	protected readonly secretKey: string;
	protected readonly logger: LoggerInterface | undefined;
	protected readonly timeoutMs: number;
	protected readonly userAgent: string;
	protected readonly fetchFn: typeof fetch;
	protected abstract readonly verifyUrl: string;
	protected abstract readonly providerName: string;

	constructor(options: HttpCaptchaProviderOptions) {
		this.secretKey = options.secretKey;
		this.logger = options.logger;
		this.timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT;
		this.userAgent = options.userAgent ?? DEFAULT_USER_AGENT;
		this.fetchFn = options.fetchFn ?? fetch;
	}

	async verify({token, remoteIp}: VerifyCaptchaParams): Promise<boolean> {
		try {
			const body = new URLSearchParams();
			body.append('secret', this.secretKey);
			body.append('response', token);
			if (remoteIp) {
				body.append('remoteip', remoteIp);
			}
			const response = await this.fetchFn(this.verifyUrl, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/x-www-form-urlencoded',
					'User-Agent': this.userAgent,
				},
				body: body.toString(),
				signal: AbortSignal.timeout(this.timeoutMs),
			});
			if (!response.ok) {
				this.logger?.error({status: response.status}, `${this.providerName} verify request failed`);
				return false;
			}
			const data = (await response.json()) as CaptchaVerifyResponse;
			if (!data.success) {
				this.logger?.warn({errorCodes: data['error-codes']}, `${this.providerName} verification failed`);
				return false;
			}
			return this.validateResponse(data);
		} catch (error) {
			if (error instanceof Error && error.name === 'TimeoutError') {
				this.logger?.error({}, `${this.providerName} verification timed out after ${this.timeoutMs}ms`);
			} else {
				this.logger?.error({error}, `Error verifying ${this.providerName} token`);
			}
			return false;
		}
	}

	protected validateResponse(_data: CaptchaVerifyResponse): boolean {
		return true;
	}
}
