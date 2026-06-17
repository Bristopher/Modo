// SPDX-License-Identifier: AGPL-3.0-or-later

import type {SearchResult} from '@fluxer/schema/src/contracts/search/SearchAdapterTypes';
import type {MessageSearchFilters, SearchableMessage} from '@fluxer/schema/src/contracts/search/SearchDocumentTypes';
import {createChannelID, createMessageID, type MessageID} from '../BrandedTypes';
import type {IMessageRepository} from '../channel/repositories/IMessageRepository';
import type {IMessageSearchService} from './IMessageSearchService';
import {deleteMessageSearchDocuments} from './MessageSearchIndexCleanup';

const RECONCILE_BATCH_SIZE = 250;

interface MessageLookupRepository {
	readonly messages: Pick<IMessageRepository, 'getMessage'>;
}

interface SearchExistingMessagesParams {
	searchService: IMessageSearchService;
	messageRepository: MessageLookupRepository;
	query: string;
	filters: MessageSearchFilters;
	hitsPerPage: number;
	page: number;
	cursor?: Array<string>;
}

interface ValidatedHits {
	validHits: Array<SearchableMessage>;
	staleMessageIds: Array<MessageID>;
}

export async function searchExistingMessages({
	searchService,
	messageRepository,
	query,
	filters,
	hitsPerPage,
	page,
	cursor,
}: SearchExistingMessagesParams): Promise<SearchResult<SearchableMessage>> {
	const result = await searchService.searchMessages(query, filters, {
		hitsPerPage,
		page: cursor?.length ? undefined : page,
		cursor,
	});
	const validated = await validateSearchHits(messageRepository, result.hits);
	if (validated.staleMessageIds.length === 0) {
		return result;
	}
	if (cursor?.length) {
		await deleteStaleSearchDocuments(searchService, validated.staleMessageIds);
		return {
			...result,
			hits: validated.validHits,
			total: Math.max(validated.validHits.length, result.total - validated.staleMessageIds.length),
		};
	}
	return reconcileOffsetSearchResult({
		searchService,
		messageRepository,
		query,
		filters,
		hitsPerPage,
		page,
	});
}

async function reconcileOffsetSearchResult({
	searchService,
	messageRepository,
	query,
	filters,
	hitsPerPage,
	page,
}: Omit<SearchExistingMessagesParams, 'cursor'>): Promise<SearchResult<SearchableMessage>> {
	const requestedOffset = (page - 1) * hitsPerPage;
	const pageHits: Array<SearchableMessage> = [];
	const staleMessageIds: Array<MessageID> = [];
	let validTotal = 0;
	let rawOffset = 0;
	let rawPage = 1;
	while (true) {
		const result = await searchService.searchMessages(query, filters, {
			hitsPerPage: RECONCILE_BATCH_SIZE,
			page: rawPage,
		});
		if (result.hits.length === 0) {
			break;
		}
		const validated = await validateSearchHits(messageRepository, result.hits);
		staleMessageIds.push(...validated.staleMessageIds);
		for (const hit of validated.validHits) {
			if (validTotal >= requestedOffset && pageHits.length < hitsPerPage) {
				pageHits.push(hit);
			}
			validTotal += 1;
		}
		rawOffset += result.hits.length;
		if (rawOffset >= result.total) {
			break;
		}
		rawPage += 1;
	}
	await deleteStaleSearchDocuments(searchService, staleMessageIds);
	return {
		hits: pageHits,
		total: validTotal,
	};
}

async function validateSearchHits(
	messageRepository: MessageLookupRepository,
	hits: Array<SearchableMessage>,
): Promise<ValidatedHits> {
	const checked = await Promise.all(
		hits.map(async (hit) => {
			let messageId: MessageID;
			try {
				const channelId = createChannelID(BigInt(hit.channelId));
				messageId = createMessageID(BigInt(hit.id));
				const message = await messageRepository.messages.getMessage(channelId, messageId);
				if (message && message.channelId.toString() === hit.channelId) {
					return {hit, staleMessageId: null};
				}
			} catch (_error) {
				try {
					messageId = createMessageID(BigInt(hit.id));
				} catch (_invalidMessageId) {
					return {hit: null, staleMessageId: null};
				}
			}
			return {hit: null, staleMessageId: messageId};
		}),
	);
	const validHits: Array<SearchableMessage> = [];
	const staleMessageIds: Array<MessageID> = [];
	for (const item of checked) {
		if (item.hit) {
			validHits.push(item.hit);
		}
		if (item.staleMessageId) {
			staleMessageIds.push(item.staleMessageId);
		}
	}
	return {validHits, staleMessageIds};
}

async function deleteStaleSearchDocuments(
	searchService: IMessageSearchService,
	messageIds: Array<MessageID>,
): Promise<void> {
	await deleteMessageSearchDocuments(messageIds, {
		searchService,
		context: {source: 'message_search_read_repair', staleMessageCount: messageIds.length},
	});
}
