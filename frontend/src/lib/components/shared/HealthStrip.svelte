<script lang="ts">
	import Icon from '@iconify/svelte';
	import { currentLocale, t } from '$lib/i18n';

	const _ = (key: string) => t($currentLocale, key);

	interface MailboxHealth {
		needs_action: number;
		waiting: number;
		done: number;
		total_messages: number;
		important: number;
		[key: string]: unknown;
	}

	interface Props {
		health: MailboxHealth;
	}

	let { health }: Props = $props();
</script>

{#if health}
	<div class="health-strip">
		<span class="health-chip needs_action"><Icon icon="tabler:alert-triangle" width="14" height="14" />{health.needs_action} {_('need action')}</span>
		<span class="health-chip waiting"><Icon icon="tabler:clock-hour-4" width="14" height="14" />{health.waiting} {_('Waiting')}</span>
		<span class="health-chip done"><Icon icon="tabler:circle-check" width="14" height="14" />{health.done} {_('Done')}</span>
		<span class="health-chip"><Icon icon="tabler:mail" width="14" height="14" />{health.total_messages} {_('total')}</span>
		{#if health.important > 0}<span class="health-chip important"><Icon icon="tabler:star" width="14" height="14" />{health.important} {_('important')}</span>{/if}
	</div>
{/if}
