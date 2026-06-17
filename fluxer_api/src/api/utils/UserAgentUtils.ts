// SPDX-License-Identifier: AGPL-3.0-or-later

import Bowser from 'bowser';
import {Logger} from '../Logger';
import {parseJsonRecord} from './JsonBoundaryUtils';

interface UserAgentInfo {
	clientOs: string;
	detectedPlatform: string;
}

type FluxerClientProperties = Record<string, unknown>;

const UNKNOWN_LABEL = 'Unknown';

function formatName(name?: string | null): string {
	const normalized = name?.trim();
	return normalized || UNKNOWN_LABEL;
}

function parseUserAgentSafe(userAgentRaw: string): UserAgentInfo {
	const ua = userAgentRaw.trim();
	if (!ua) return {clientOs: UNKNOWN_LABEL, detectedPlatform: UNKNOWN_LABEL};
	try {
		const parser = Bowser.getParser(ua);
		return {
			clientOs: formatName(parser.getOSName()),
			detectedPlatform: formatName(parser.getBrowserName()),
		};
	} catch (error) {
		Logger.warn({error}, 'Failed to parse user agent');
		return {clientOs: UNKNOWN_LABEL, detectedPlatform: UNKNOWN_LABEL};
	}
}

export function isFluxerFlutterClient(request: Request): boolean {
	const parsed = parseFluxerClientProperties(request);
	return parsed?.client_runtime === 'flutter';
}

export function isFluxerFlutterAndroidClient(request: Request): boolean {
	return isFluxerFlutterPlatformClient(request, 'android');
}

export function isFluxerFlutterIosClient(request: Request): boolean {
	return (
		isFluxerFlutterPlatformClient(request, 'ios') ||
		isFluxerFlutterPlatformClient(request, 'iphone') ||
		isFluxerFlutterPlatformClient(request, 'ipad') ||
		isFluxerFlutterPlatformClient(request, 'ipod')
	);
}

function isFluxerFlutterPlatformClient(request: Request, platform: string): boolean {
	const platformCandidates = getFluxerFlutterPlatformCandidates(request);
	if (!platformCandidates) return false;
	const needle = platform.toLowerCase();
	return platformCandidates.some((value) => typeof value === 'string' && value.toLowerCase().includes(needle));
}

function getFluxerFlutterPlatformCandidates(request: Request): Array<unknown> | null {
	const parsed = parseFluxerClientProperties(request);
	if (parsed?.client_runtime !== 'flutter') return null;
	return [
		parsed.client_os,
		parsed.os,
		parsed.operating_system,
		parsed.platform,
		parsed.device_platform,
		parsed.client_platform,
		parsed.user_agent,
		request.headers.get('user-agent'),
	];
}

function parseFluxerClientProperties(request: Request): FluxerClientProperties | null {
	const encoded = request.headers.get('x-fluxer-client-properties');
	if (!encoded) return null;
	try {
		const decoded = Buffer.from(encoded, 'base64').toString('utf8');
		return parseJsonRecord(decoded);
	} catch {
		return null;
	}
}

export function resolveSessionClientInfo(args: {userAgent: string | null; isDesktopClient: boolean | null}): {
	clientOs: string;
	clientPlatform: string;
} {
	const parsed = parseUserAgentSafe(args.userAgent ?? '');
	const clientPlatform = args.isDesktopClient ? 'Fluxer Desktop' : parsed.detectedPlatform;
	return {
		clientOs: parsed.clientOs,
		clientPlatform,
	};
}
