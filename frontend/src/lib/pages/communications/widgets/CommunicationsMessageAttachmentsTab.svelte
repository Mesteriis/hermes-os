<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { CommunicationAttachment } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	interface Props {
		selectedAttachments: CommunicationAttachment[];
		attachmentIcon: (contentType: string) => string;
		formatBytes: (bytes: number) => string;
	}

	let { selectedAttachments, attachmentIcon, formatBytes }: Props = $props();
</script>

{#each selectedAttachments as attachment}
	<article class="attachment-bubble">
		<Icon icon={attachmentIcon(attachment.content_type)} width="34" height="34" />
		<span>
			<strong>{attachment.filename ?? attachment.provider_attachment_id}</strong>
			<small>{formatBytes(attachment.size_bytes)} · {attachment.content_type} · {attachment.scan_status}</small>
		</span>
		<button type="button" disabled><Icon icon="tabler:download" width="16" height="16" /></button>
	</article>
{:else}
	<div class="empty-panel">{_('No attachments')}</div>
{/each}
