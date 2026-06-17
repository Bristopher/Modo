// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {msg} from '@lingui/core/macro';
import {useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const TEXT_TO_SPEECH_NOT_SUPPORTED_DESCRIPTOR = msg({
	message: 'Text-to-speech not supported',
	comment: 'Title of the modal shown when the browser does not support text-to-speech.',
});
const TEXT_TO_SPEECH_IS_NOT_SUPPORTED_BY_YOUR_DESCRIPTOR = msg({
	message:
		"Text-to-speech is not supported by your browser. Some browsers like Brave block this feature for privacy reasons. Try using Chrome, Firefox, or Edge, or adjust your browser's privacy settings.",
	comment:
		'Body of the TTS-unsupported modal. Mentions specific browsers; keep Chrome, Firefox, Edge, and Brave as literal proper nouns.',
});
export const TtsUnsupportedModal = observer(() => {
	const {i18n} = useLingui();
	return (
		<ConfirmModal
			title={i18n._(TEXT_TO_SPEECH_NOT_SUPPORTED_DESCRIPTOR)}
			description={i18n._(TEXT_TO_SPEECH_IS_NOT_SUPPORTED_BY_YOUR_DESCRIPTOR)}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="voice.tts-unsupported-modal.confirm-modal"
		/>
	);
});
