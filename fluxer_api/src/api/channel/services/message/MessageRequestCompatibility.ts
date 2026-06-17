// SPDX-License-Identifier: AGPL-3.0-or-later

export function normalizeMessageRequestPayload(payload: unknown): unknown {
	if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
		return payload;
	}
	const normalized = {...(payload as Record<string, unknown>)};
	if (!('embeds' in normalized) && 'embed' in normalized) {
		const embed = normalized.embed;
		normalized.embeds = embed == null ? [] : [embed];
	}
	delete normalized.embed;
	return normalized;
}
