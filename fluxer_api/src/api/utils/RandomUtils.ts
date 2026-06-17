// SPDX-License-Identifier: AGPL-3.0-or-later

import crypto from 'node:crypto';

const RANDOM_STRING_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
const RANDOM_DIGIT_ALPHABET = '0123456789';

export function randomNumericCode(length: number): string {
	const alphabetLength = RANDOM_DIGIT_ALPHABET.length;
	const rangeSize = 256 - (256 % alphabetLength);
	const randomBytes = new Uint8Array(length * 2);
	crypto.getRandomValues(randomBytes);
	let result = '';
	let byteIndex = 0;
	while (result.length < length) {
		if (byteIndex >= randomBytes.length) {
			crypto.getRandomValues(randomBytes);
			byteIndex = 0;
		}
		const randomByte = randomBytes[byteIndex++]!;
		if (randomByte >= rangeSize) continue;
		result += RANDOM_DIGIT_ALPHABET.charAt(randomByte % alphabetLength);
	}
	return result;
}

export function randomString(length: number) {
	const alphabetLength = RANDOM_STRING_ALPHABET.length;
	const rangeSize = 256 - (256 % alphabetLength);
	const randomBytes = new Uint8Array(length * 2);
	crypto.getRandomValues(randomBytes);
	let result = '';
	let byteIndex = 0;
	while (result.length < length) {
		if (byteIndex >= randomBytes.length) {
			crypto.getRandomValues(randomBytes);
			byteIndex = 0;
		}
		const randomByte = randomBytes[byteIndex++];
		if (randomByte >= rangeSize) {
			continue;
		}
		result += RANDOM_STRING_ALPHABET.charAt(randomByte % alphabetLength);
	}
	return result;
}
