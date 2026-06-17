// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const COMMUNITY_AT_CAPACITY_DESCRIPTOR = msg({
	message: 'Community at capacity',
	comment: 'Short label in the guild at capacity modal. Keep it concise.',
});
const THIS_COMMUNITY_HAS_REACHED_ITS_MAXIMUM_MEMBER_LIMIT_DESCRIPTOR = msg({
	message: 'This community has reached its maximum member limit and is not accepting new members at this time.',
	comment: 'Description text in the guild at capacity modal.',
});
export const GuildAtCapacityModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(COMMUNITY_AT_CAPACITY_DESCRIPTOR)}
			description={i18n._(THIS_COMMUNITY_HAS_REACHED_ITS_MAXIMUM_MEMBER_LIMIT_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="guild.guild-at-capacity-modal.confirm-modal"
		/>
	);
});
