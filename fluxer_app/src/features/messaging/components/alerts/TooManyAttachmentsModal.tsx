// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import Users from '@app/features/user/state/Users';
import {msg, plural} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const WHOA_THIS_IS_HEAVY_DESCRIPTOR = msg({
	message: 'Whoa, this is heavy',
	comment: 'Label in the too many attachments modal.',
});
export const TooManyAttachmentsModal = observer(() => {
	const {i18n} = useLingui();
	const currentUser = Users.currentUser;
	const maxAttachments = currentUser?.maxAttachmentsPerMessage ?? 10;
	return (
		<ConfirmModal
			title={i18n._(WHOA_THIS_IS_HEAVY_DESCRIPTOR)}
			description={plural(
				{count: maxAttachments},
				{
					one: 'You can only upload # file at a time. Try again with fewer files.',
					other: 'You can only upload # files at a time. Try again with fewer files.',
				},
			)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="messaging.too-many-attachments-modal.confirm-modal"
		/>
	);
});
