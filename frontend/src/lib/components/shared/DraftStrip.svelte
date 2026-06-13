<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';
	import './draftStrip.css';

	const _ = (key: string) => t($currentLocale, key);

	interface Draft {
		draft_id: string;
		account_id: string;
		to_recipients: string[];
		cc_recipients: string[];
		subject: string;
		body_text: string;
		[key: string]: unknown;
	}

	interface ComposeForm {
		draft_id: string;
		account_id: string;
		to_text: string;
		cc_text: string;
		subject: string;
		body: string;
	}

	interface Props {
		drafts: Draft[];
		onOpenCompose: (draft: Draft) => void;
		onDeleteDraft?: (draftId: string) => void | Promise<void>;
		deleteDraft?: (draftId: string) => void | Promise<void>;
	}

	let { drafts, onOpenCompose, onDeleteDraft, deleteDraft }: Props = $props();
	let stripElement = $state<HTMLDivElement | null>(null);

	function handleDraftActionClick(event: MouseEvent): void {
		if (!(event.target instanceof Element)) return;
		const button = event.target.closest('[data-draft-action]') as HTMLButtonElement | null;
		if (!button || (stripElement && !stripElement.contains(button))) return;
		const draftId = button.dataset.draftId;
		if (!draftId) return;
		event.preventDefault();
		event.stopPropagation();

		if (button.dataset.draftAction === 'delete') {
			void (deleteDraft ?? onDeleteDraft)?.(draftId);
			return;
		}

		const draft = drafts.find((candidate) => candidate.draft_id === draftId);
		if (draft) onOpenCompose(draft);
	}

	$effect(() => {
		const element = stripElement;
		if (!element) return;
		element.addEventListener('pointerdown', handleDraftActionClick);
		element.addEventListener('click', handleDraftActionClick);
		return () => {
			element.removeEventListener('pointerdown', handleDraftActionClick);
			element.removeEventListener('click', handleDraftActionClick);
		};
	});
</script>

{#if drafts.length > 0}
	<div class="draft-strip" bind:this={stripElement}>
		<strong>{_('Drafts')} ({drafts.length})</strong>
		{#each drafts.slice(0, 3) as draft}
			<div class="draft-chip">
				<button
					type="button"
					class="draft-open-button"
					data-draft-action="open"
					data-draft-id={draft.draft_id}
					onpointerdown={handleDraftActionClick}
					onclick={handleDraftActionClick}
				>
					<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject || _('Untitled draft')}
				</button>
				<button
					type="button"
					class="draft-delete-button"
					data-draft-action="delete"
					data-draft-id={draft.draft_id}
					onpointerdown={handleDraftActionClick}
					onclick={handleDraftActionClick}
					aria-label={`${_('Delete draft')}: ${draft.subject || _('Untitled draft')}`}
				>
					<Icon icon="tabler:trash" width="14" height="14" />
				</button>
			</div>
		{/each}
	</div>
{/if}
