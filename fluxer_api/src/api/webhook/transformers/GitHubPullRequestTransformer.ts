// SPDX-License-Identifier: AGPL-3.0-or-later

import type {RichEmbedRequest} from '@fluxer/schema/src/domains/message/MessageRequestSchemas';
import type {GitHubWebhook} from '@fluxer/schema/src/domains/webhook/GitHubWebhookSchemas';
import {parseString} from '../../utils/StringUtils';

export async function transformPullRequest(body: GitHubWebhook): Promise<RichEmbedRequest | null> {
	if (!(body.pull_request && body.action && body.repository)) {
		return null;
	}
	const authorIconUrl = body.pull_request.user.avatar_url;
	const authorName = body.pull_request.user.login;
	const authorUrl = body.pull_request.user.html_url;
	const repoName = body.repository.full_name;
	const prNumber = body.pull_request.number;
	const prTitle = body.pull_request.title;
	const prUrl = body.pull_request.html_url;
	const prDescription = body.pull_request.body || '';
	let title: string;
	let color: number;
	switch (body.action) {
		case 'opened': {
			title = `[${repoName}] Pull request opened: #${prNumber} ${prTitle}`;
			color = 0x098efc;
			break;
		}
		case 'closed': {
			title = `[${repoName}] Pull request closed: #${prNumber} ${prTitle}`;
			color = 0x000000;
			break;
		}
		case 'reopened': {
			title = `[${repoName}] Pull request reopened: #${prNumber} ${prTitle}`;
			color = 0xfcbd1f;
			break;
		}
		default:
			return null;
	}
	return {
		title: parseString(title, 70),
		url: prUrl,
		color,
		description: body.action === 'opened' ? parseString(prDescription, 350) : undefined,
		author: {
			name: authorName,
			url: authorUrl,
			icon_url: authorIconUrl,
		},
	};
}

export async function transformPullRequestReview(body: GitHubWebhook): Promise<RichEmbedRequest | null> {
	if (!body.review || body.action !== 'submitted' || !body.pull_request || !body.repository) {
		return null;
	}
	const authorIconUrl = body.review.user.avatar_url;
	const authorName = body.review.user.login;
	const authorUrl = body.review.user.html_url;
	const repoName = body.repository.full_name;
	const prNumber = body.pull_request.number;
	const prTitle = body.pull_request.title;
	const reviewUrl = body.review.html_url;
	const reviewBody = body.review.body || 'No description provided';
	const title = `[${repoName}] Pull request review submitted: #${prNumber} ${prTitle}`;
	const color = 0x000000;
	return {
		title: parseString(title, 70),
		url: reviewUrl,
		color,
		description: parseString(reviewBody, 350),
		author: {
			name: authorName,
			url: authorUrl,
			icon_url: authorIconUrl,
		},
	};
}

export async function transformPullRequestReviewComment(body: GitHubWebhook): Promise<RichEmbedRequest | null> {
	if (!body.comment || body.action !== 'created' || !body.pull_request || !body.repository) {
		return null;
	}
	const authorIconUrl = body.comment.user.avatar_url;
	const authorName = body.comment.user.login;
	const authorUrl = body.comment.user.html_url;
	const repoName = body.repository.full_name;
	const prNumber = body.pull_request.number;
	const prTitle = body.pull_request.title;
	const commentUrl = body.comment.html_url;
	const commentBody = body.comment.body || 'No description provided';
	const title = `[${repoName}] New review comment on pull request #${prNumber}: ${prTitle}`;
	const color = 0xc00a7f;
	return {
		title: parseString(title, 70),
		url: commentUrl,
		color,
		description: parseString(commentBody, 350),
		author: {
			name: authorName,
			url: authorUrl,
			icon_url: authorIconUrl,
		},
	};
}
