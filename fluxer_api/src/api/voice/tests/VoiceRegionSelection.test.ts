// SPDX-License-Identifier: AGPL-3.0-or-later

import {describe, expect, it} from 'vitest';
import type {VoiceRegionAvailability, VoiceServerRecord} from '../VoiceModel';
import {
	resolveVoiceRegionPreference,
	selectClosestPseudoRegionServer,
	selectVoiceRegionId,
} from '../VoiceRegionSelection';

function createRegionAvailability({
	id,
	latitude,
	longitude,
	isDefault,
}: {
	id: string;
	latitude: number;
	longitude: number;
	isDefault: boolean;
}): VoiceRegionAvailability {
	return {
		id,
		name: `Region ${id.toUpperCase()}`,
		emoji: id.toUpperCase(),
		latitude,
		longitude,
		isDefault,
		vipOnly: false,
		requiredGuildFeatures: [],
		isAccessible: true,
		restrictions: {
			vipOnly: false,
			requiredGuildFeatures: new Set(),
			allowedGuildIds: new Set(),
			allowedUserIds: new Set(),
		},
		serverCount: 1,
		activeServerCount: 1,
	};
}

function createVoiceServer({
	regionId,
	serverId,
	latitude,
	longitude,
}: {
	regionId: string;
	serverId: string;
	latitude: number | null;
	longitude: number | null;
}): VoiceServerRecord {
	return {
		regionId,
		serverId,
		endpoint: `wss://${serverId}.voice.example.com`,
		apiKey: `${serverId}-key`,
		apiSecret: `${serverId}-secret`,
		latitude,
		longitude,
		isActive: true,
		restrictions: {
			vipOnly: false,
			requiredGuildFeatures: new Set(),
			allowedGuildIds: new Set(),
			allowedUserIds: new Set(),
		},
		createdAt: null,
		updatedAt: null,
	};
}

describe('VoiceRegionSelection', () => {
	it('selects the closest region when coordinates are provided', () => {
		const regions = [
			createRegionAvailability({id: 'a', latitude: 0, longitude: 0, isDefault: true}),
			createRegionAvailability({id: 'b', latitude: 50, longitude: 50, isDefault: false}),
		];
		const preference = resolveVoiceRegionPreference({
			preferredRegionId: null,
			accessibleRegions: regions,
			availableRegions: regions,
			defaultRegionId: null,
		});
		const selected = selectVoiceRegionId({
			preferredRegionId: preference.regionId,
			mode: preference.mode,
			accessibleRegions: regions,
			availableRegions: regions,
			latitude: '49',
			longitude: '49',
			selectionKey: 'guild:1:channel:1',
		});
		expect(selected).toBe('b');
	});
	it('keeps explicit regions even when coordinates would choose another', () => {
		const regions = [
			createRegionAvailability({id: 'a', latitude: 0, longitude: 0, isDefault: false}),
			createRegionAvailability({id: 'b', latitude: 50, longitude: 50, isDefault: false}),
		];
		const preference = resolveVoiceRegionPreference({
			preferredRegionId: 'a',
			accessibleRegions: regions,
			availableRegions: regions,
			defaultRegionId: null,
		});
		const selected = selectVoiceRegionId({
			preferredRegionId: preference.regionId,
			mode: preference.mode,
			accessibleRegions: regions,
			availableRegions: regions,
			latitude: '49',
			longitude: '49',
			selectionKey: 'guild:1:channel:1',
		});
		expect(preference.mode).toBe('explicit');
		expect(selected).toBe('a');
	});
	it('selects the closest pseudo-region server when server coordinates are configured', () => {
		const serverA = createVoiceServer({regionId: 'a', serverId: 'a1', latitude: 0, longitude: 0});
		const serverB = createVoiceServer({regionId: 'b', serverId: 'b1', latitude: 51, longitude: 51});
		const selectedServer = selectClosestPseudoRegionServer({
			mode: 'automatic',
			accessibleServers: [serverA, serverB],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		expect(selectedServer?.serverId).toBe('b1');
		expect(selectedServer?.regionId).toBe('b');
	});
	it('balances pseudo-region server ties independently of input order', () => {
		const serverA = createVoiceServer({regionId: 'a', serverId: 'a1', latitude: 51, longitude: 51});
		const serverB = createVoiceServer({regionId: 'b', serverId: 'b1', latitude: 51, longitude: 51});
		const selectedFromForwardOrder = selectClosestPseudoRegionServer({
			mode: 'automatic',
			accessibleServers: [serverB, serverA],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		const selectedFromReverseOrder = selectClosestPseudoRegionServer({
			mode: 'automatic',
			accessibleServers: [serverA, serverB],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		const selectedForAnotherRoom = selectClosestPseudoRegionServer({
			mode: 'automatic',
			accessibleServers: [serverB, serverA],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:2',
		});
		expect(selectedFromForwardOrder?.serverId).toBe('b1');
		expect(selectedFromReverseOrder?.serverId).toBe('b1');
		expect(selectedForAnotherRoom?.serverId).toBe('a1');
	});
	it('does not use pseudo-region servers during explicit selection mode', () => {
		const serverA = createVoiceServer({regionId: 'a', serverId: 'a1', latitude: 0, longitude: 0});
		const serverB = createVoiceServer({regionId: 'b', serverId: 'b1', latitude: 51, longitude: 51});
		const selectedServer = selectClosestPseudoRegionServer({
			mode: 'explicit',
			accessibleServers: [serverA, serverB],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		expect(selectedServer).toBeNull();
	});
	it('balances closest-region ties independently of input order', () => {
		const regions = [
			createRegionAvailability({id: 'a', latitude: 51, longitude: 51, isDefault: false}),
			createRegionAvailability({id: 'b', latitude: 51, longitude: 51, isDefault: false}),
		];
		const selectedFromForwardOrder = selectVoiceRegionId({
			preferredRegionId: null,
			mode: 'automatic',
			accessibleRegions: [regions[1], regions[0]],
			availableRegions: [regions[1], regions[0]],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		const selectedFromReverseOrder = selectVoiceRegionId({
			preferredRegionId: null,
			mode: 'automatic',
			accessibleRegions: [regions[0], regions[1]],
			availableRegions: [regions[0], regions[1]],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:1',
		});
		const selectedForAnotherRoom = selectVoiceRegionId({
			preferredRegionId: null,
			mode: 'automatic',
			accessibleRegions: [regions[1], regions[0]],
			availableRegions: [regions[1], regions[0]],
			latitude: '50',
			longitude: '50',
			selectionKey: 'guild:1:channel:2',
		});
		expect(selectedFromForwardOrder).toBe('b');
		expect(selectedFromReverseOrder).toBe('b');
		expect(selectedForAnotherRoom).toBe('a');
	});
});
