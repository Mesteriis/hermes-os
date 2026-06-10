<script lang="ts">
	import Icon from '@iconify/svelte';

	interface Props {
		activeCommunicationEmptySection: {
			icon: string;
			label: string;
		} | null;
	}

	let { activeCommunicationEmptySection }: Props = $props();
</script>

{#if activeCommunicationEmptySection}
	<section class="communications-page communication-empty-page">
		<div class="view-header">
			<div class="view-title-with-icon">
				<span class="hero-mark small"><Icon icon={activeCommunicationEmptySection.icon} width="28" height="28" /></span>
				<div>
					<h1>{activeCommunicationEmptySection.label}</h1>
					<p>This Communications workspace is ready in navigation, but no data contract is implemented yet.</p>
				</div>
			</div>
		</div>
		<section class="panel communication-empty-panel">
			<Icon icon={activeCommunicationEmptySection.icon} width="42" height="42" />
			<div>
				<h2>{activeCommunicationEmptySection.label}</h2>
				<p>Hermes Hub will keep this as a second-level Communications context. It is intentionally empty until a scoped backend and UI slice defines real records for this workspace.</p>
			</div>
		</section>
	</section>
{/if}
