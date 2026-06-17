// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ApiContext, ApiServices, RequestScope} from './ApiContext';
import {CassandraPhoneLookupRepository} from './auth/services/PhoneLookupRepository';
import {Config} from './Config';
import {getInboundSmsChallengeServiceInstance} from './middleware/ServiceMiddleware';
import {
	getGatewayService,
	getKVClient,
	getMediaService,
	getSnowflakeService,
	getWorkerService,
} from './middleware/ServiceRegistry';
import {
	getBotMfaMirrorService,
	getCacheService,
	getContactChangeLogService,
	getEmailDnsValidationService,
	getEmailService,
	getPhoneAttemptRiskService,
	getRateLimitService,
	getSmsService,
	getUserActivityBuffer,
	getUserRepository,
} from './middleware/ServiceSingletons';

let cachedServices: ApiServices | null = null;
let cachedConfigRef: typeof Config | null = null;

function buildApiServices(): ApiServices {
	if (cachedServices && cachedConfigRef === Config) {
		return cachedServices;
	}
	cachedServices = {
		users: getUserRepository(),
		cache: getCacheService(),
		gateway: getGatewayService(),
		kv: getKVClient(),
		media: getMediaService(),
		email: getEmailService(),
		emailDnsValidation: getEmailDnsValidationService(),
		sms: getSmsService(),
		worker: getWorkerService(),
		snowflake: getSnowflakeService(),
		rateLimit: getRateLimitService(),
		contactChangeLog: getContactChangeLogService(),
		inboundSmsChallenge: getInboundSmsChallengeServiceInstance(),
		phoneLookup: new CassandraPhoneLookupRepository(),
		phoneAttemptRisk: getPhoneAttemptRiskService(),
		botMfaMirror: getBotMfaMirrorService(),
		userActivityBuffer: getUserActivityBuffer(),
		config: Config,
	};
	cachedConfigRef = Config;
	return cachedServices;
}

export function resetApiServicesForTesting(): void {
	cachedServices = null;
	cachedConfigRef = null;
}

const ANONYMOUS_REQUEST_SCOPE: RequestScope = {
	requestId: 'anonymous',
	clientIp: null,
	userAgent: null,
};

export function createApiContext(scope: RequestScope = ANONYMOUS_REQUEST_SCOPE): ApiContext {
	return {
		services: buildApiServices(),
		request: scope,
	};
}
