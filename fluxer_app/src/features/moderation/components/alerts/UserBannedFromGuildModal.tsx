// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const YOU_RE_BANNED_DESCRIPTOR = msg({
	message: "You're banned",
	comment: 'Short label in the user banned from guild modal. Keep it concise. Keep the tone plain and specific.',
});
const YOU_ARE_BANNED_FROM_THIS_COMMUNITY_AND_CANNOT_DESCRIPTOR = msg({
	message: 'You are banned from this community and cannot join.',
	comment: 'Error message in the user banned from guild modal. Keep the tone plain and specific.',
});
export const UserBannedFromGuildModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(YOU_RE_BANNED_DESCRIPTOR)}
			description={i18n._(YOU_ARE_BANNED_FROM_THIS_COMMUNITY_AND_CANNOT_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="moderation.user-banned-from-guild-modal.confirm-modal"
		/>
	);
});
