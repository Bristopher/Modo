// SPDX-License-Identifier: AGPL-3.0-or-later

import {decode} from 'html-entities';

export function decodeHTMLEntities(html?: string | null): string {
	if (!html) return '';
	return decode(html);
}

export function stripHtmlTags(html?: string | null): string {
	if (!html) return '';
	return html.replace(/<[^>]*>/g, '');
}

export function htmlToMarkdown(html?: string | null): string {
	if (!html) return '';
	let md = html
		.replace(/<p>/gi, '\n\n')
		.replace(/<\/p>/gi, '')
		.replace(/<br\s*\/?>/gi, '\n')
		.replace(/<h[1-6]>/gi, '\n\n**')
		.replace(/<\/h[1-6]>/gi, '**\n\n')
		.replace(/<li>/gi, '• ')
		.replace(/<\/li>/gi, '\n')
		.replace(/<ul>|<ol>/gi, '\n')
		.replace(/<\/ul>|<\/ol>/gi, '\n')
		.replace(/<pre><code>([\s\S]*?)<\/code><\/pre>/gi, (_, code) => `\`\`\`\n${code}\n\`\`\``)
		.replace(/<code>([\s\S]*?)<\/code>/gi, '`$1`')
		.replace(/<strong>([\s\S]*?)<\/strong>/gi, '**$1**')
		.replace(/<b>([\s\S]*?)<\/b>/gi, '**$1**')
		.replace(/<em>([\s\S]*?)<\/em>/gi, '_$1_')
		.replace(/<i>([\s\S]*?)<\/i>/gi, '_$1_')
		.replace(/<a\s+(?:[^>]*?\s+)?href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi, '[$2]($1)');
	md = stripHtmlTags(md);
	md = decodeHTMLEntities(md);
	return md
		.replace(/\n{3,}/g, '\n\n')
		.replace(/\s+$/, '')
		.trim();
}
