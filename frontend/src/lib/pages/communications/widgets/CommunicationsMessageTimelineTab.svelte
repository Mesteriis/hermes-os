<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { CommunicationMessageDetailItem, CommunicationMessageSummary } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type DetailMessage = CommunicationMessageDetailItem | CommunicationMessageSummary;

	interface Props {
		selectedMessage: DetailMessage;
		messageTime: (msg: unknown) => string;
	}

	let { selectedMessage, messageTime }: Props = $props();
</script>

<div class="timeline-list">
	<p><Icon icon="tabler:circle-dot" width="14" height="14" />{_('Received')} · {messageTime(selectedMessage)}</p>
	<p><Icon icon="tabler:circle-dot" width="14" height="14" />{_('Projected locally')} · {messageTime(selectedMessage)}</p>
</div>
