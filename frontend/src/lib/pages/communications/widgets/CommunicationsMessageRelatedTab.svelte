<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import type { RelatedCommunicationMessage } from '$lib/services/communications';
	import type { CommunicationMessageDetailItem, CommunicationMessageSummary, MailMessageInsight } from '$lib/api';

	const _ = (key: string) => t($currentLocale, key);

	type DetailMessage = CommunicationMessageDetailItem | CommunicationMessageSummary;

	interface Props {
		selectedMessage: DetailMessage;
		relatedMessages: RelatedCommunicationMessage[];
		mailMessageInsight: MailMessageInsight | null;
		isMailActionRunning: boolean;
		importantActionIcon: string;
		importantActionLabel: string;
		messageTime: (msg: unknown) => string;
		senderLabel: (sender: string) => string;
		onSelectRelatedMessage: (messageId: string) => void;
		onPin: () => void;
		onImportant: () => void;
		onMute: () => void;
		onTrash: () => void;
		onRestore: () => void;
		onSnooze: () => void;
		onExport: (format: 'md' | 'eml' | 'json') => void;
	}

	let {
		selectedMessage,
		relatedMessages,
		mailMessageInsight,
		isMailActionRunning,
		importantActionIcon,
		importantActionLabel,
		messageTime,
		senderLabel,
		onSelectRelatedMessage,
		onPin,
		onImportant,
		onMute,
		onTrash,
		onRestore,
		onSnooze,
		onExport
	}: Props = $props();
</script>

{#if relatedMessages.length}
	<section class="related-link-list">
		<h3>{_('Related conversations')}</h3>
		{#each relatedMessages as relatedMessage}
			<button type="button" onclick={() => onSelectRelatedMessage(relatedMessage.message_id)}>
				<span>
					<Icon icon={relatedMessage.relation === 'same_conversation' ? 'tabler:messages' : 'tabler:user-circle'} width="15" height="15" />
					{relatedMessage.relation === 'same_conversation' ? _('Same conversation') : _('Same contact')}
				</span>
				<strong>{relatedMessage.subject}</strong>
				<small>{senderLabel(relatedMessage.sender)} · {messageTime(relatedMessage)}</small>
			</button>
		{/each}
	</section>
{/if}
<div class="related-workspace">
	<button type="button" onclick={onPin} disabled={isMailActionRunning}><Icon icon="tabler:pin" width="16" height="16" />{_('Pin')}</button>
	<button type="button" onclick={onImportant} disabled={isMailActionRunning}><Icon icon={importantActionIcon} width="16" height="16" />{_(importantActionLabel)}</button>
	<button type="button" onclick={onMute} disabled={isMailActionRunning}><Icon icon="tabler:volume-off" width="16" height="16" />{_('Mute')}</button>
	{#if selectedMessage.local_state === 'trash'}
		<button type="button" onclick={onRestore} disabled={isMailActionRunning}><Icon icon="tabler:restore" width="16" height="16" />{_('Restore')}</button>
	{:else}
		<button type="button" onclick={onTrash} disabled={isMailActionRunning}><Icon icon="tabler:trash" width="16" height="16" />{_('Delete')}</button>
	{/if}
	<button type="button" onclick={onSnooze} disabled={isMailActionRunning}><Icon icon="tabler:alarm-snooze" width="16" height="16" />{_('Snooze')}</button>
	<button type="button" onclick={() => onExport('md')} disabled={isMailActionRunning}><Icon icon="tabler:file-export" width="16" height="16" />MD</button>
	<button type="button" onclick={() => onExport('eml')} disabled={isMailActionRunning}><Icon icon="tabler:mail-forward" width="16" height="16" />EML</button>
	<button type="button" onclick={() => onExport('json')} disabled={isMailActionRunning}><Icon icon="tabler:braces" width="16" height="16" />JSON</button>
</div>
{#if mailMessageInsight?.aiReply?.body}
	<article class="mail-result-card"><strong>{mailMessageInsight.aiReply.subject ?? _('AI Reply')}</strong><p>{mailMessageInsight.aiReply.body}</p></article>
{/if}
{#if mailMessageInsight?.tasks.length}
	<article class="mail-result-card"><strong>{_('Extracted Tasks')}</strong>{#each mailMessageInsight.tasks as task}<p>{task.title}</p>{/each}</article>
{/if}
{#if mailMessageInsight?.notes.length}
	<article class="mail-result-card"><strong>{_('Extracted Notes')}</strong>{#each mailMessageInsight.notes as note}<p>{note.title}: {note.content}</p>{/each}</article>
{/if}
