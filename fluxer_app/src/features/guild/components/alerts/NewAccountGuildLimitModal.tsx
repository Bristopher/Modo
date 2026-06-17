// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {OKAY_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const SLOW_DOWN_DESCRIPTOR = msg({
	message: 'Slow down',
	comment: 'Short label in the new account guild limit modal. Keep it concise.',
});
const NEW_ACCOUNTS_ARE_LIMITED_TO_JOINING_10_COMMUNITIES_DESCRIPTOR = msg({
	message: 'New accounts can join up to 10 communities in the first 24 hours. Try again later.',
	comment: 'Description text in the new account guild limit modal.',
});
export const NewAccountGuildLimitModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(SLOW_DOWN_DESCRIPTOR)}
			description={i18n._(NEW_ACCOUNTS_ARE_LIMITED_TO_JOINING_10_COMMUNITIES_DESCRIPTOR)}
			primaryText={i18n._(OKAY_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="guild.new-account-guild-limit-modal.confirm-modal"
		/>
	);
});
