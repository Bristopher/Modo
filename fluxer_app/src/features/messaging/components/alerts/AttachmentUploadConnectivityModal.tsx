// SPDX-License-Identifier: AGPL-3.0-or-later

import {ConfirmModal} from '@app/features/app/components/dialogs/ConfirmModal';
import {ExternalLink} from '@app/features/app/components/shared/ExternalLink';
import {PRODUCT_NAME, SUPPORT_EMAIL, SUPPORT_EMAIL_MAILTO} from '@app/features/app/config/I18nDisplayConstants';
import {UNDERSTOOD_DESCRIPTOR} from '@app/features/i18n/utils/CommonMessageDescriptors';
import {MULTIPART_ATTACHMENT_FALLBACK_MAX_REQUEST_SIZE} from '@app/features/messaging/utils/AttachmentUploadFallbackUtils';
import {formatFileSize} from '@app/features/messaging/utils/FileUtils';
import {msg} from '@lingui/core/macro';
import {Trans, useLingui} from '@lingui/react/macro';
import {observer} from 'mobx-react-lite';

const ATTACHMENT_UPLOAD_UNAVAILABLE_DESCRIPTOR = msg({
	message: 'Attachment upload unavailable',
	comment: 'Short label in the attachment upload connectivity modal. Keep it concise.',
});
export const AttachmentUploadConnectivityModal = observer(() => {
	const {i18n} = useLingui();
	const fallbackSizeFormatted = formatFileSize(MULTIPART_ATTACHMENT_FALLBACK_MAX_REQUEST_SIZE);
	return (
		<ConfirmModal
			title={i18n._(ATTACHMENT_UPLOAD_UNAVAILABLE_DESCRIPTOR)}
			description={
				<Trans>
					We couldn't reach the attachment upload service. {PRODUCT_NAME} can fall back to the standard upload path for
					messages up to {fallbackSizeFormatted} total, but these attachments exceed that combined limit and can't be
					sent until connectivity is restored. Try again later or contact{' '}
					<ExternalLink
						href={SUPPORT_EMAIL_MAILTO}
						data-flx="messaging.attachment-upload-connectivity-modal.external-link"
					>
						{SUPPORT_EMAIL}
					</ExternalLink>{' '}
					if this keeps happening.
				</Trans>
			}
			primaryText={i18n._(UNDERSTOOD_DESCRIPTOR)}
			onPrimary={() => {}}
			secondaryText={false}
			hideCloseButton
			data-flx="messaging.attachment-upload-connectivity-modal.confirm-modal"
		/>
	);
});
