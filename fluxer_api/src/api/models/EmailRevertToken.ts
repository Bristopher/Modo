// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';
import {createEmailRevertToken} from '../BrandedTypes';
import type {EmailRevertTokenRow} from '../database/types/AuthTypes';

export class EmailRevertToken {
	readonly token: string;
	readonly userId: UserID;
	readonly email: string;

	constructor(row: EmailRevertTokenRow) {
		this.token = row.token_;
		this.userId = row.user_id;
		this.email = row.email;
	}

	toRow(): EmailRevertTokenRow {
		return {
			token_: createEmailRevertToken(this.token),
			user_id: this.userId,
			email: this.email,
		};
	}
}
