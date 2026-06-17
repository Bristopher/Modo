// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {PRODUCT_NAME} from '@app/features/app/config/I18nDisplayConstants';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const INVITES_PAUSED_DESCRIPTOR = msg({
	message: 'Invites paused',
	comment: 'Button or menu action label in the invites disabled modal. Keep it concise.',
});
const DETECTED_A_POTENTIAL_RAID_IN_THIS_COMMUNITY_SO_DESCRIPTOR = msg({
	message: '{productName} spotted a possible raid here. Invites are paused — try again later.',
	comment: 'Label in the invites disabled modal.',
});
const COMMUNITY_ADMINS_HAVE_PAUSED_INVITES_SO_YOU_CAN_DESCRIPTOR = msg({
	message: "Admins paused invites — you can't join right now.",
	comment: 'Error message in the invites disabled modal.',
});

interface InvitesDisabledModalProps {
	isRaidDetected?: boolean;
}

export const InvitesDisabledModal = observer(({isRaidDetected = false}: InvitesDisabledModalProps) => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(INVITES_PAUSED_DESCRIPTOR)}
			description={
				isRaidDetected
					? i18n._(DETECTED_A_POTENTIAL_RAID_IN_THIS_COMMUNITY_SO_DESCRIPTOR, {productName: PRODUCT_NAME})
					: i18n._(COMMUNITY_ADMINS_HAVE_PAUSED_INVITES_SO_YOU_CAN_DESCRIPTOR)
			}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="invite.invites-disabled-modal.confirm-modal"
		/>
	);
});
