// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const YOUR_IP_IS_BANNED_DESCRIPTOR = msg({
	message: 'Your IP is banned',
	comment: 'Label in the user ip banned from guild modal. Keep the tone plain and specific.',
});
const YOUR_IP_ADDRESS_IS_BANNED_FROM_THIS_COMMUNITY_DESCRIPTOR = msg({
	message: 'Your IP address is banned from this community and you cannot join.',
	comment: 'Error message in the user ip banned from guild modal. Keep the tone plain and specific.',
});
export const UserIpBannedFromGuildModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(YOUR_IP_IS_BANNED_DESCRIPTOR)}
			description={i18n._(YOUR_IP_ADDRESS_IS_BANNED_FROM_THIS_COMMUNITY_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="moderation.user-ip-banned-from-guild-modal.confirm-modal"
		/>
	);
});
