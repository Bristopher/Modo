// SPDX-License-Identifier: AGPL-3.0-or-later

import {ms} from 'itty-time';

export interface AttachmentDecayRules {
	minMb: number;
	maxMb: number;
	maxEligibleMb: number;
	minDays: number;
	maxDays: number;
	curve: number;
	pricePerTBPerMonth?: number;
}

interface AttachmentDecayInput {
	sizeBytes: bigint | number;
	uploadedAt: Date;
	rules?: AttachmentDecayRules;
}

interface AttachmentDecayResult {
	expiresAt: Date;
	days: number;
	cost: number;
}

export const DEFAULT_DECAY_CONSTANTS = {
	MIN_MB: 5,
	MAX_MB: 500,
	MIN_DAYS: 14,
	MAX_DAYS: 365 * 3,
	PLAN_MB: 500,
	CURVE: 0.5,
	PRICE_PER_TB_PER_MONTH: 0.0081103 * 1000,
};
export const DEFAULT_RENEWAL_CONSTANTS = {
	RENEW_THRESHOLD_DAYS: 30,
	RENEW_WINDOW_DAYS: 30,
	MIN_WINDOW_DAYS: 7,
	MAX_WINDOW_DAYS: 30,
	MIN_THRESHOLD_DAYS: 3,
	MAX_THRESHOLD_DAYS: 14,
};

function toMb(sizeBytes: bigint | number): number {
	const n = typeof sizeBytes === 'bigint' ? Number(sizeBytes) : sizeBytes;
	return n / 1024 / 1024;
}

export function computeDecay({sizeBytes, uploadedAt, rules}: AttachmentDecayInput): AttachmentDecayResult | null {
	const constants = {
		minMb: rules?.minMb ?? DEFAULT_DECAY_CONSTANTS.MIN_MB,
		maxMb: rules?.maxMb ?? DEFAULT_DECAY_CONSTANTS.MAX_MB,
		maxEligibleMb: rules?.maxEligibleMb ?? DEFAULT_DECAY_CONSTANTS.PLAN_MB,
		minDays: rules?.minDays ?? DEFAULT_DECAY_CONSTANTS.MIN_DAYS,
		maxDays: rules?.maxDays ?? DEFAULT_DECAY_CONSTANTS.MAX_DAYS,
		curve: rules?.curve ?? DEFAULT_DECAY_CONSTANTS.CURVE,
		pricePerTBPerMonth: rules?.pricePerTBPerMonth ?? DEFAULT_DECAY_CONSTANTS.PRICE_PER_TB_PER_MONTH,
	};
	const sizeMB = toMb(sizeBytes);
	if (sizeMB > constants.maxEligibleMb) return null;
	let lifetimeDays: number;
	if (sizeMB <= constants.minMb) {
		lifetimeDays = constants.maxDays;
	} else if (sizeMB >= constants.maxMb) {
		lifetimeDays = constants.minDays;
	} else {
		const linearFrac = (sizeMB - constants.minMb) / (constants.maxMb - constants.minMb);
		const logFrac = Math.log(sizeMB / constants.minMb) / Math.log(constants.maxMb / constants.minMb);
		const blend = (1 - constants.curve) * linearFrac + constants.curve * logFrac;
		lifetimeDays = constants.maxDays - blend * (constants.maxDays - constants.minDays);
	}
	const expiresAt = new Date(uploadedAt);
	expiresAt.setUTCDate(expiresAt.getUTCDate() + lifetimeDays);
	const sizeTB = (typeof sizeBytes === 'bigint' ? Number(sizeBytes) : sizeBytes) / 1024 / 1024 / 1024 / 1024;
	const lifetimeMonths = lifetimeDays / 30;
	const cost = sizeTB * constants.pricePerTBPerMonth * lifetimeMonths;
	return {
		expiresAt,
		cost,
		days: Math.round(lifetimeDays),
	};
}

const MS_PER_DAY = ms('1 day');

export function computeCost({
	sizeBytes,
	lifetimeDays,
	pricePerTBPerMonth = DEFAULT_DECAY_CONSTANTS.PRICE_PER_TB_PER_MONTH,
}: {
	sizeBytes: bigint | number;
	lifetimeDays: number;
	pricePerTBPerMonth?: number;
}): number {
	const sizeTB = (typeof sizeBytes === 'bigint' ? Number(sizeBytes) : sizeBytes) / 1024 / 1024 / 1024 / 1024;
	const lifetimeMonths = lifetimeDays / 30;
	return sizeTB * pricePerTBPerMonth * lifetimeMonths;
}

export function getExpiryBucket(expiresAt: Date): number {
	return Number(
		`${expiresAt.getUTCFullYear()}${String(expiresAt.getUTCMonth() + 1).padStart(2, '0')}${String(expiresAt.getUTCDate()).padStart(2, '0')}`,
	);
}

export function extendExpiry(currentExpiry: Date | null, newlyComputed: Date): Date {
	if (!currentExpiry) return newlyComputed;
	return currentExpiry > newlyComputed ? currentExpiry : newlyComputed;
}

export function maybeRenewExpiry({
	currentExpiry,
	now,
	thresholdDays = DEFAULT_RENEWAL_CONSTANTS.RENEW_THRESHOLD_DAYS,
	windowDays = DEFAULT_RENEWAL_CONSTANTS.RENEW_WINDOW_DAYS,
	maxExpiry,
}: {
	currentExpiry: Date | null;
	now: Date;
	thresholdDays?: number;
	windowDays?: number;
	maxExpiry?: Date;
}): Date | null {
	if (!currentExpiry) return null;
	if (windowDays <= 0) return null;
	const remainingMs = currentExpiry.getTime() - now.getTime();
	if (remainingMs > thresholdDays * MS_PER_DAY) {
		return null;
	}
	const targetMs = now.getTime() + windowDays * MS_PER_DAY;
	const cappedTargetMs = maxExpiry ? Math.min(maxExpiry.getTime(), targetMs) : targetMs;
	if (cappedTargetMs <= currentExpiry.getTime()) {
		return null;
	}
	const target = new Date(now);
	target.setTime(cappedTargetMs);
	return target;
}
