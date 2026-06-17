// SPDX-License-Identifier: AGPL-3.0-or-later

import {maskIpForDisplay} from '@fluxer/ip_utils/src/IpAddress';
import type {AuthSessionResponse} from '@fluxer/schema/src/domains/auth/AuthSchemas';
import {uint8ArrayToBase64} from 'uint8array-extras';
import {Config} from '../Config';
import {Logger} from '../Logger';
import type {AuthSession} from '../models/AuthSession';
import {getLocationLabelFromIp} from '../utils/IpUtils';
import {resolveSessionClientInfo} from '../utils/UserAgentUtils';

const DEV_FALLBACK_AUTH_SESSION_LOCATION = 'Stockholm, Stockholm County, Sweden';

function shouldUseFallbackAuthSessionLocation(): boolean {
	return Config.dev.testModeEnabled || Config.nodeEnv === 'development';
}

async function resolveAuthSessionLocation(session: AuthSession): Promise<string | null> {
	try {
		const location = await getLocationLabelFromIp(session.clientIp);
		return location ?? (shouldUseFallbackAuthSessionLocation() ? DEV_FALLBACK_AUTH_SESSION_LOCATION : null);
	} catch (error) {
		Logger.warn({error, clientIp: session.clientIp}, 'Failed to resolve location from IP');
		return shouldUseFallbackAuthSessionLocation() ? DEV_FALLBACK_AUTH_SESSION_LOCATION : null;
	}
}

export async function mapAuthSessionsToResponse({
	authSessions,
	currentSessionId,
}: {
	authSessions: Array<AuthSession>;
	currentSessionId?: Uint8Array;
}): Promise<Array<AuthSessionResponse>> {
	const sortedSessions = authSessions.toSorted((a, b) => {
		const aTime = a.approximateLastUsedAt?.getTime() || 0;
		const bTime = b.approximateLastUsedAt?.getTime() || 0;
		return bTime - aTime;
	});
	const locationResults = await Promise.allSettled(
		sortedSessions.map((session) => resolveAuthSessionLocation(session)),
	);
	return sortedSessions.map((authSession, index): AuthSessionResponse => {
		const locationResult = locationResults[index];
		const clientLocation = locationResult?.status === 'fulfilled' ? locationResult.value : null;
		let clientOs: string;
		let clientPlatform: string;
		if (authSession.clientUserAgent) {
			const parsed = resolveSessionClientInfo({
				userAgent: authSession.clientUserAgent,
				isDesktopClient: authSession.clientIsDesktop,
			});
			clientOs = parsed.clientOs;
			clientPlatform = parsed.clientPlatform;
		} else {
			clientOs = authSession.clientOs || 'Unknown';
			clientPlatform = authSession.clientPlatform || 'Unknown';
		}
		const idHash = uint8ArrayToBase64(authSession.sessionIdHash, {urlSafe: true});
		const isCurrent = currentSessionId ? Buffer.compare(authSession.sessionIdHash, currentSessionId) === 0 : false;
		return {
			id_hash: idHash,
			client_info: {
				platform: clientPlatform,
				os: clientOs,
				browser: undefined,
				location: clientLocation
					? {
							city: clientLocation.split(',').at(0)?.trim() || null,
							region: clientLocation.split(',').at(1)?.trim() || null,
							country: clientLocation.split(',').at(2)?.trim() || null,
						}
					: null,
			},
			masked_ip: maskIpForDisplay(authSession.clientIp),
			approx_last_used_at: authSession.approximateLastUsedAt?.toISOString() || null,
			current: isCurrent,
		};
	});
}
