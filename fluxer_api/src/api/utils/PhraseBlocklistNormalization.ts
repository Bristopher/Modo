// SPDX-License-Identifier: AGPL-3.0-or-later

import {transliterate} from 'transliteration';

const CONTROL_OR_FORMAT_REGEX = /[\p{Cc}\p{Cf}\uFFFE\uFFFF]/gu;
const VARIATION_SELECTOR_REGEX = /(?:[\uFE00-\uFE0F]|[\u{E0100}-\u{E01EF}])/gu;
const COMBINING_MARKS_REGEX = /\p{M}+/gu;
const NON_ALPHANUMERIC_REGEX = /[^\p{L}\p{N}]+/gu;
const ASCII_NON_ALPHANUMERIC_REGEX = /[^a-z0-9]+/g;
const WHITESPACE_REGEX = /\s+/gu;
const MIN_AGGRESSIVE_FORM_LENGTH = 3;

interface PhraseMatchForms {
	raw: string;
	words: string;
	compact: string;
	asciiWords: string;
	asciiCompact: string;
}

function collapseWhitespace(value: string): string {
	return value.replace(WHITESPACE_REGEX, ' ').trim();
}

function stripIgnorableCharacters(value: string): string {
	return value.replace(CONTROL_OR_FORMAT_REGEX, '').replace(VARIATION_SELECTOR_REGEX, '');
}

function maybeKeepAggressiveForm(value: string): string {
	return value.length >= MIN_AGGRESSIVE_FORM_LENGTH ? value : '';
}

function buildWordFormFromCanonical(value: string): string {
	return collapseWhitespace(
		value.normalize('NFKD').replace(COMBINING_MARKS_REGEX, '').replace(NON_ALPHANUMERIC_REGEX, ' '),
	);
}

function buildAsciiWordFormFromCanonical(value: string): string {
	return collapseWhitespace(
		transliterate(value)
			.toLowerCase()
			.normalize('NFKD')
			.replace(COMBINING_MARKS_REGEX, '')
			.replace(ASCII_NON_ALPHANUMERIC_REGEX, ' '),
	);
}

export function canonicalizeStoredPhrase(value: string): string {
	return stripIgnorableCharacters(value.normalize('NFKC')).toLowerCase().trim();
}

export function buildPhraseMatchForms(value: string): PhraseMatchForms {
	const raw = canonicalizeStoredPhrase(value);
	const words = maybeKeepAggressiveForm(buildWordFormFromCanonical(raw));
	const compact = maybeKeepAggressiveForm(words.replace(WHITESPACE_REGEX, ''));
	const asciiWords = maybeKeepAggressiveForm(buildAsciiWordFormFromCanonical(raw));
	const asciiCompact = maybeKeepAggressiveForm(asciiWords.replace(WHITESPACE_REGEX, ''));
	return {
		raw,
		words,
		compact,
		asciiWords,
		asciiCompact,
	};
}
