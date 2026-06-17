// SPDX-License-Identifier: AGPL-3.0-or-later

import type {ReadStateResponse} from '@fluxer/schema/src/domains/gateway/GatewaySchemas';
import {encodeReadStateProtoNative} from '@fluxer/schema/src/domains/read_state/ReadStateProtoCodec';
import type {ReadState} from '../models/ReadState';

export function mapReadStateResponse(readState: ReadState): ReadStateResponse {
	return {
		id: readState.channelId.toString(),
		mention_count: readState.mentionCount,
		last_message_id: readState.lastMessageId?.toString() ?? null,
		last_pin_timestamp: readState.lastPinTimestamp?.toISOString() ?? null,
		version: readState.version.toString(),
	};
}

export function encodeReadStatesResponseProto(readStates: ReadonlyArray<ReadState>): string {
	return encodeReadStateProtoNative(
		readStates.map((readState) => ({
			channelId: readState.channelId,
			mentionCount: readState.mentionCount,
			lastMessageId: readState.lastMessageId,
			lastPinTimestamp: readState.lastPinTimestamp,
			version: readState.version,
		})),
	);
}
