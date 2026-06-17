// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import type {User} from '@app/features/user/models/User';
import {msg, plural} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const TOO_MANY_COMMUNITIES_DESCRIPTOR = msg({
	message: 'Too many communities',
	comment: 'Error message in the max guilds modal.',
});

interface MaxGuildsModalProps {
	user: User;
}

export const MaxGuildsModal = observer(({user}: MaxGuildsModalProps) => {
	const {i18n} = useLingui();
	const maxGuilds = user.maxGuilds;
	return (
		<ConfirmModal
			title={i18n._(TOO_MANY_COMMUNITIES_DESCRIPTOR)}
			description={plural(
				{count: maxGuilds},
				{
					one: "You've reached the maximum number of communities you can join (# community). Leave a community before joining another one.",
					other:
						"You've reached the maximum number of communities you can join (# communities). Leave a community before joining another one.",
				},
			)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="guild.max-guilds-modal.confirm-modal"
		/>
	);
});
