// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const WHOA_THIS_IS_HEAVY_DESCRIPTOR = msg({
	message: 'Whoa, this is heavy',
	comment: 'Label in the too many reactions modal.',
});
const THIS_IS_ONE_HEAVY_MESSAGE_SOME_REACTIONS_NEED_DESCRIPTOR = msg({
	message: 'This is one heavy message. Some reactions need to be removed before you can add more.',
	comment: 'Description text in the too many reactions modal. Keep the tone plain and specific.',
});
export const TooManyReactionsModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(WHOA_THIS_IS_HEAVY_DESCRIPTOR)}
			description={i18n._(THIS_IS_ONE_HEAVY_MESSAGE_SOME_REACTIONS_NEED_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="messaging.too-many-reactions-modal.confirm-modal"
		/>
	);
});
