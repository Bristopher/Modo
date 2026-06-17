// SPDX-License-Identifier: AGPL-3.0-or-later

import {randomInt} from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import {fileURLToPath} from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let tails: Array<string> | undefined;
let scales: Array<string> | undefined;

function getTails(): Array<string> {
	if (!tails) {
		initWords();
	}
	return tails!;
}

function getScales(): Array<string> {
	if (!scales) {
		initWords();
	}
	return scales!;
}

export function generateConnectionId(): string {
	const scaleWords = getScales();
	const tailWords = getTails();
	const scale = scaleWords[randomInt(scaleWords.length)];
	const tail = tailWords[randomInt(tailWords.length)];
	return `${tail}-${scale}`;
}

function initWords(): void {
	const wordsDir = path.join(__dirname);
	tails = parseWordsFile(path.join(wordsDir, 'tails.txt'));
	scales = parseWordsFile(path.join(wordsDir, 'scales.txt'));
}

function parseWordsFile(filePath: string): Array<string> {
	const content = fs.readFileSync(filePath, 'utf-8');
	const lines = content.split('\n');
	const words: Array<string> = [];
	for (const line of lines) {
		const trimmed = line.trim();
		if (trimmed && !trimmed.startsWith('#')) {
			words.push(trimmed);
		}
	}
	return words;
}
