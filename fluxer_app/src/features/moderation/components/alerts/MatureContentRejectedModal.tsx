// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const MATURE_CONTENT_NOT_ALLOWED_DESCRIPTOR = msg({
	message: 'Mature content not allowed',
	comment: 'Error message in the mature content rejected modal.',
});
const THIS_CHANNEL_IS_NOT_MARKED_FOR_MATURE_CONTENT_DESCRIPTOR = msg({
	message:
		'This channel is not marked for mature content. Mature content can only be sent in channels marked for mature content. Ask a moderator to update this channel if appropriate.',
	comment: 'Label in the mature content rejected modal.',
});
export const MatureContentRejectedModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(MATURE_CONTENT_NOT_ALLOWED_DESCRIPTOR)}
			description={i18n._(THIS_CHANNEL_IS_NOT_MARKED_FOR_MATURE_CONTENT_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="moderation.mature-content-rejected-modal.confirm-modal"
		/>
	);
});
