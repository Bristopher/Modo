// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LoggerInterface} from '@fluxer/logger/src/LoggerInterface';
import type {ISmsProvider} from '@pkgs/sms/src/providers/ISmsProvider';
import {TestSmsProvider} from '@pkgs/sms/src/providers/TestSmsProvider';
import {TwilioSmsProvider, type TwilioSmsProviderConfig} from '@pkgs/sms/src/providers/TwilioSmsProvider';
import {UnavailableSmsProvider} from '@pkgs/sms/src/providers/UnavailableSmsProvider';

interface BaseSmsProviderFactoryParams {
	logger?: LoggerInterface;
}

interface CreateUnavailableSmsProviderParams extends BaseSmsProviderFactoryParams {
	mode: 'unavailable';
}

interface CreateTestSmsProviderParams extends BaseSmsProviderFactoryParams {
	mode: 'test';
	verificationCode?: string;
}

interface CreateTwilioSmsProviderParams extends BaseSmsProviderFactoryParams {
	mode: 'twilio';
	config: TwilioSmsProviderConfig;
	fetchFn?: typeof fetch;
}

type CreateSmsProviderParams =
	| CreateUnavailableSmsProviderParams
	| CreateTestSmsProviderParams
	| CreateTwilioSmsProviderParams;

export function createSmsProvider(params: CreateSmsProviderParams): ISmsProvider {
	if (params.mode === 'test') {
		return new TestSmsProvider({
			logger: params.logger,
			verificationCode: params.verificationCode,
		});
	}
	if (params.mode === 'twilio') {
		return new TwilioSmsProvider({
			config: params.config,
			logger: params.logger,
			fetchFn: params.fetchFn,
		});
	}
	return new UnavailableSmsProvider();
}
