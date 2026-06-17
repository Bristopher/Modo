// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const ROLE_NAME_CANNOT_BE_BLANK_DESCRIPTOR = msg({
	message: 'Role name is required',
	comment: 'Validation modal title when the role-edit form is submitted with an empty role name.',
});
const YOU_CANNOT_SAVE_A_ROLE_WITH_A_BLANK_DESCRIPTOR = msg({
	message: 'Give the role a name before saving.',
	comment: 'Modal body shown when the role-edit form is submitted with an empty role name.',
});
export const RoleNameBlankModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(ROLE_NAME_CANNOT_BE_BLANK_DESCRIPTOR)}
			description={i18n._(YOU_CANNOT_SAVE_A_ROLE_WITH_A_BLANK_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="app.role-name-blank-modal.confirm-modal"
		/>
	);
});
