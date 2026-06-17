// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {OKAY_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const YOU_RE_DELETING_MESSAGES_TOO_QUICKLY_DESCRIPTOR = msg({
	message: "You're deleting messages too quickly",
	comment: 'Label in the message delete too quick modal. Keep the tone plain and specific.',
});
const WAIT_A_MOMENT_BEFORE_DELETING_MORE_MESSAGES_DESCRIPTOR = msg({
	message: 'Wait a moment before deleting more messages.',
	comment: 'Description text in the message delete too quick modal. Keep the tone plain and specific.',
});
export const MessageDeleteTooQuickModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(YOU_RE_DELETING_MESSAGES_TOO_QUICKLY_DESCRIPTOR)}
			description={i18n._(WAIT_A_MOMENT_BEFORE_DELETING_MORE_MESSAGES_DESCRIPTOR)}
			primaryText={i18n._(OKAY_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="messaging.message-delete-too-quick-modal.confirm-modal"
		/>
	);
});
