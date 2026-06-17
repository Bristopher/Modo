// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ICacheService} from '@pkgs/cache/src/ICacheService';
import type {IEmailService} from '@pkgs/email/src/IEmailService';
import type {IKVProvider} from '@pkgs/kv_client/src/IKVProvider';
import type {IRateLimitService} from '@pkgs/rate_limit/src/IRateLimitService';
import type {ISmsService} from '@pkgs/sms/src/ISmsService';
import type {IWorkerService} from '@pkgs/worker/src/contracts/IWorkerService';
import type {InboundSmsChallengeService} from './auth/services/InboundSmsChallengeService';
import type {PhoneAttemptRiskService} from './auth/services/PhoneAttemptRiskService';
import type {IPhoneLookupRepository} from './auth/services/PhoneLookupRepository';
import type {Config} from './Config';
import type {IEmailDnsValidationService} from './infrastructure/IEmailDnsValidationService';
import type {IGatewayService} from './infrastructure/IGatewayService';
import type {IMediaService} from './infrastructure/IMediaService';
import type {ISnowflakeService} from './infrastructure/ISnowflakeService';
import type {BotMfaMirrorService} from './oauth/BotMfaMirrorService';
import type {IUserRepository} from './user/IUserRepository';
import type {UserActivityBuffer} from './user/services/UserActivityBuffer';
import type {UserContactChangeLogService} from './user/services/UserContactChangeLogService';
import type {WorkerTaskName} from './worker/WorkerLaneConfig';

export interface ApiServices {
	users: IUserRepository;
	cache: ICacheService;
	gateway: IGatewayService;
	kv: IKVProvider;
	media: IMediaService;
	email: IEmailService;
	emailDnsValidation: IEmailDnsValidationService;
	sms: ISmsService;
	worker: IWorkerService<WorkerTaskName>;
	snowflake: ISnowflakeService;
	rateLimit: IRateLimitService;
	contactChangeLog: UserContactChangeLogService;
	inboundSmsChallenge: InboundSmsChallengeService | null;
	phoneLookup: IPhoneLookupRepository | null;
	phoneAttemptRisk: PhoneAttemptRiskService;
	botMfaMirror: BotMfaMirrorService;
	userActivityBuffer: UserActivityBuffer;
	config: typeof Config;
}

export interface RequestScope {
	requestId: string;
	clientIp: string | null;
	userAgent: string | null;
}

export interface ApiContext {
	services: ApiServices;
	request: RequestScope;
}
