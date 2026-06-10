<script lang="ts">
	import Icon from '@iconify/svelte';

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
	}

	let { drafts, onOpenCompose }: Props = $props();
</script>

{#if drafts.length > 0}
	<div class="draft-strip">
		<strong>Drafts ({drafts.length})</strong>
		{#each drafts.slice(0, 3) as draft}
			<button type="button" class="draft-chip" onclick={() => onOpenCompose(draft)}>
				<Icon icon="tabler:pencil" width="14" height="14" />{draft.subject}
			</button>
		{/each}
	</div>
{/if}
