// SPDX-License-Identifier: AGPL-3.0-or-later

export interface EmailConfig {
	enabled: boolean;
	fromEmail: string;
	fromName: string;
	appBaseUrl: string;
	marketingBaseUrl: string;
}

export interface EmailMessage {
	to: string;
	from: {
		email: string;
		name: string;
	};
	subject: string;
	text: string;
}

export interface IEmailProvider {
	sendEmail(message: EmailMessage): Promise<boolean>;
}

export interface UserBouncedEmailChecker {
	isEmailBounced(email: string): Promise<boolean>;
}
