// SPDX-License-Identifier: AGPL-3.0-or-later

import type {IChannelDataRepository} from './IChannelDataRepository';
import type {IMessageInteractionRepository} from './IMessageInteractionRepository';
import type {IMessageRepository} from './IMessageRepository';

export abstract class IChannelRepositoryAggregate {
	abstract readonly channelData: IChannelDataRepository;
	abstract readonly messages: IMessageRepository;
	abstract readonly messageInteractions: IMessageInteractionRepository;
}
