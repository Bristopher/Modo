// SPDX-License-Identifier: AGPL-3.0-or-later

import type {UserID} from '../BrandedTypes';
import {createEmailVerificationToken} from '../BrandedTypes';
import type {EmailVerificationTokenRow} from '../database/types/AuthTypes';

export class EmailVerificationToken {
	readonly token: string;
	readonly userId: UserID;
	readonly email: string;

	constructor(row: EmailVerificationTokenRow) {
		this.token = row.token_;
		this.userId = row.user_id;
		this.email = row.email;
	}

	toRow(): EmailVerificationTokenRow {
		return {
			token_: createEmailVerificationToken(this.token),
			user_id: this.userId,
			email: this.email,
		};
	}
}
