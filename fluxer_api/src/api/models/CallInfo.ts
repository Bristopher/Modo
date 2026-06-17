// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';
import type {MessageCall} from '../database/types/MessageTypes';

export class CallInfo {
	readonly participantIds: Set<UserID>;
	readonly endedTimestamp: Date | null;

	constructor(call: MessageCall) {
		this.participantIds = call.participant_ids ?? new Set();
		this.endedTimestamp = call.ended_timestamp ? new Date(call.ended_timestamp) : null;
	}

	toMessageCall(): MessageCall {
		return {
			participant_ids: this.participantIds.size > 0 ? this.participantIds : null,
			ended_timestamp: this.endedTimestamp,
		};
	}
}
