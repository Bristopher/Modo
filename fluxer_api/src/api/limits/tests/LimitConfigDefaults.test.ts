// SPDX-License-Identifier: AGPL-3.0-or-later

import type {LimitConfigSnapshot, LimitRule} from '@fluxer/limits/src/LimitTypes';
import {describe, expect, test} from 'vitest';
import {createDefaultLimitConfig, mergeWithCurrentDefaults} from '../../constants/LimitConfig';

interface LegacyLimitRule extends LimitRule {
	unlockedFeatures?: Array<string>;
}

interface LegacyLimitConfigSnapshot extends Omit<LimitConfigSnapshot, 'rules'> {
	rules: Array<LegacyLimitRule>;
}

describe('Limit config defaults', () => {
	test('hosted defaults include only premium and default tier limit rules', () => {
		const config = createDefaultLimitConfig({selfHosted: false});
		const premiumRule = config.rules.find((rule) => rule.id === 'premium');
		const defaultRule = config.rules.find((rule) => rule.id === 'default');
		expect(premiumRule).toBeDefined();
		expect(defaultRule).toBeDefined();
		expect(config.rules.map((rule) => rule.id)).toEqual(['premium', 'default']);
	});
	test('self-hosted defaults include only default tier limit rule', () => {
		const config = createDefaultLimitConfig({selfHosted: true});
		expect(config.rules.map((rule) => rule.id)).toEqual(['default']);
	});
});

describe('Limit config default merge', () => {
	test('legacy unlocked features on known rules are dropped during merge', () => {
		const legacyConfig: LegacyLimitConfigSnapshot = {
			traitDefinitions: ['premium'],
			rules: [
				{
					id: 'premium',
					filters: {traits: ['premium']},
					limits: {},
					unlockedFeatures: ['MORE_EMOJI', 'UNLIMITED_EMOJI'],
				},
				{
					id: 'default',
					limits: {},
				},
			],
		};
		const merged = mergeWithCurrentDefaults(legacyConfig, {selfHosted: false});
		const premiumRule = merged.rules.find((rule) => rule.id === 'premium') as Record<string, unknown> | undefined;
		expect(premiumRule?.unlockedFeatures).toBeUndefined();
	});
});
