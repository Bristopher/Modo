// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IpInfoLookupResult} from '@pkgs/geoip/src/IpInfoService';
import {afterAll, beforeAll, beforeEach, describe, expect, it} from 'vitest';
import {AdminRepository} from '../../admin/AdminRepository';
import type {ApiTestHarness} from '../../test/ApiTestHarness';
import {createApiTestHarness} from '../../test/ApiTestHarness';
import {
	drainAbuseAutoBanTasksForTests,
	hashAuthToken,
	recordAbuseSignal,
	resetAbuseTrackingForTests,
} from '../AbusiveIpAutoBanner';
import {ipBanCache} from '../IpBanMiddleware';
import {setInjectedIpInfoService} from '../ServiceMiddleware';

function ipInfoResult(ip: string, overrides: Partial<IpInfoLookupResult> = {}): IpInfoLookupResult {
	return {
		ip,
		available: true,
		riskNote: 'test',
		geo: {
			countryCode: 'US',
			countryName: 'United States',
			continent: 'North America',
			continentCode: 'NA',
			region: null,
			regionCode: null,
			city: null,
			postalCode: null,
			timezone: null,
			latitude: null,
			longitude: null,
			accuracyRadiusKm: null,
		},
		asn: {
			asn: 'AS64500',
			number: 64500,
			name: 'Test ISP',
			domain: null,
			type: null,
		},
		mobile: {
			name: null,
			mcc: null,
			mnc: null,
		},
		anonymous: {
			isAnonymous: false,
			providerName: null,
			isVpn: false,
			isProxy: false,
			isResidentialProxy: false,
			isTor: false,
			isRelay: false,
			percentDaysSeen: null,
		},
		flags: {
			isAnycast: false,
			isHosting: false,
			isMobile: false,
			isSatellite: false,
		},
		...overrides,
	};
}

async function waitForAssertion(assertion: () => void): Promise<void> {
	const deadline = Date.now() + 1000;
	let lastError: unknown;
	while (Date.now() < deadline) {
		try {
			assertion();
			return;
		} catch (err) {
			lastError = err;
			await new Promise((resolve) => setTimeout(resolve, 10));
		}
	}
	if (lastError) throw lastError;
	assertion();
}

describe('AbusiveIpAutoBanner', () => {
	let harness: ApiTestHarness;
	let adminRepository: AdminRepository;
	beforeAll(async () => {
		harness = await createApiTestHarness();
		adminRepository = new AdminRepository();
	});
	beforeEach(async () => {
		await harness.reset();
		resetAbuseTrackingForTests();
		ipBanCache.resetCaches();
		setInjectedIpInfoService({
			async lookup(ip: string) {
				return ipInfoResult(ip);
			},
		});
	});
	afterAll(async () => {
		setInjectedIpInfoService(undefined);
		await harness.shutdown();
	});
	it('temporarily bans an IP that tries many distinct invalid tokens', async () => {
		const ip = '8.8.8.8';
		for (let i = 0; i < 10; i += 1) {
			recordAbuseSignal(ip, 'auth_failure:session', {tokenHash: hashAuthToken(`invalid-${i}`)});
		}
		await waitForAssertion(() => {
			expect(ipBanCache.isBanned(ip)).toBe(true);
		});
		await drainAbuseAutoBanTasksForTests();
		expect(ipBanCache.getMatch(ip)?.kind).toBe('temporary_24h');
		await expect(adminRepository.isIpBanned(ip)).resolves.toBe(true);
	});
	it('does not auto-ban after a single score-only spike', async () => {
		const ip = '8.8.4.4';
		recordAbuseSignal(ip, 'http_429', {weight: 150});
		await new Promise((resolve) => setTimeout(resolve, 25));
		await drainAbuseAutoBanTasksForTests();
		expect(ipBanCache.isBanned(ip)).toBe(false);
		await expect(adminRepository.isIpBanned(ip)).resolves.toBe(false);
	});
	it('raises token-diversity tolerance for mobile carrier IPs', async () => {
		const ip = '1.1.1.1';
		setInjectedIpInfoService({
			async lookup(candidateIp: string) {
				return ipInfoResult(candidateIp, {
					mobile: {name: 'Test Mobile', mcc: '001', mnc: '01'},
					flags: {isAnycast: false, isHosting: false, isMobile: true, isSatellite: false},
				});
			},
		});
		for (let i = 0; i < 10; i += 1) {
			recordAbuseSignal(ip, 'auth_failure:session', {tokenHash: hashAuthToken(`mobile-invalid-${i}`)});
		}
		await new Promise((resolve) => setTimeout(resolve, 25));
		await drainAbuseAutoBanTasksForTests();
		expect(ipBanCache.isBanned(ip)).toBe(false);
		await expect(adminRepository.isIpBanned(ip)).resolves.toBe(false);
	});
	it('does not auto-ban mobile carrier IPs even at mobile token-diversity threshold', async () => {
		const ip = '1.0.0.1';
		setInjectedIpInfoService({
			async lookup(candidateIp: string) {
				return ipInfoResult(candidateIp, {
					mobile: {name: 'Test Mobile', mcc: '001', mnc: '01'},
					flags: {isAnycast: false, isHosting: false, isMobile: true, isSatellite: false},
				});
			},
		});
		for (let i = 0; i < 100; i += 1) {
			recordAbuseSignal(ip, 'auth_failure:session', {tokenHash: hashAuthToken(`mobile-threshold-${i}`)});
		}
		await new Promise((resolve) => setTimeout(resolve, 25));
		await drainAbuseAutoBanTasksForTests();
		expect(ipBanCache.isBanned(ip)).toBe(false);
		await expect(adminRepository.isIpBanned(ip)).resolves.toBe(false);
	});
	it('does not auto-ban loopback or private IP addresses', async () => {
		for (const ip of ['127.0.0.1', '10.0.0.10', '::ffff:127.0.0.1']) {
			for (let i = 0; i < 20; i += 1) {
				recordAbuseSignal(ip, 'auth_failure:session', {tokenHash: hashAuthToken(`${ip}-invalid-${i}`)});
			}
			await drainAbuseAutoBanTasksForTests();
			expect(ipBanCache.isBanned(ip)).toBe(false);
			await expect(adminRepository.isIpBanned(ip)).resolves.toBe(false);
		}
	});
});
