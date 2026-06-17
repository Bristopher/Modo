// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {ICaptchaProvider} from '@pkgs/captcha/src/ICaptchaProvider';
import {HcaptchaProvider} from '@pkgs/captcha/src/providers/HcaptchaProvider';
import type {HttpCaptchaProviderOptions} from '@pkgs/captcha/src/providers/HttpCaptchaProvider';
import type {RecaptchaProviderOptions} from '@pkgs/captcha/src/providers/RecaptchaProvider';
import {RecaptchaProvider} from '@pkgs/captcha/src/providers/RecaptchaProvider';
import {TestCaptchaProvider} from '@pkgs/captcha/src/providers/TestProvider';
import {TurnstileProvider} from '@pkgs/captcha/src/providers/TurnstileProvider';
import {UnavailableCaptchaProvider} from '@pkgs/captcha/src/providers/UnavailableCaptchaProvider';

interface BaseCaptchaProviderFactoryParams {
	logger?: LoggerInterface;
}

interface CreateUnavailableCaptchaProviderParams extends BaseCaptchaProviderFactoryParams {
	mode: 'unavailable';
}

interface CreateTestCaptchaProviderParams extends BaseCaptchaProviderFactoryParams {
	mode: 'test';
}

interface CreateHcaptchaProviderParams extends BaseCaptchaProviderFactoryParams {
	mode: 'hcaptcha';
	secretKey: string;
	timeoutMs?: number;
	userAgent?: string;
	fetchFn?: typeof fetch;
}

interface CreateTurnstileProviderParams extends BaseCaptchaProviderFactoryParams {
	mode: 'turnstile';
	secretKey: string;
	timeoutMs?: number;
	userAgent?: string;
	fetchFn?: typeof fetch;
}

interface CreateRecaptchaProviderParams extends BaseCaptchaProviderFactoryParams {
	mode: 'recaptcha';
	secretKey: string;
	minimumScore?: number;
	timeoutMs?: number;
	userAgent?: string;
	fetchFn?: typeof fetch;
}

type CreateCaptchaProviderParams =
	| CreateUnavailableCaptchaProviderParams
	| CreateTestCaptchaProviderParams
	| CreateHcaptchaProviderParams
	| CreateTurnstileProviderParams
	| CreateRecaptchaProviderParams;

function buildHttpOptions(
	params: CreateHcaptchaProviderParams | CreateTurnstileProviderParams | CreateRecaptchaProviderParams,
): HttpCaptchaProviderOptions {
	return {
		secretKey: params.secretKey,
		logger: params.logger,
		timeoutMs: params.timeoutMs,
		userAgent: params.userAgent,
		fetchFn: params.fetchFn,
	};
}

export function createCaptchaProvider(params: CreateCaptchaProviderParams): ICaptchaProvider {
	if (params.mode === 'test') {
		return new TestCaptchaProvider();
	}
	if (params.mode === 'hcaptcha') {
		return new HcaptchaProvider(buildHttpOptions(params));
	}
	if (params.mode === 'turnstile') {
		return new TurnstileProvider(buildHttpOptions(params));
	}
	if (params.mode === 'recaptcha') {
		const options: RecaptchaProviderOptions = {
			...buildHttpOptions(params),
			minimumScore: params.minimumScore,
		};
		return new RecaptchaProvider(options);
	}
	return new UnavailableCaptchaProvider();
}
