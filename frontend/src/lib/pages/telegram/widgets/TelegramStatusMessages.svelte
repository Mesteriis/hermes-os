<script lang="ts">
	import '$lib/components/shared/setupControls.css';
	interface Props {
		actionMessage: string;
		error: string;
	}

	let { actionMessage, error }: Props = $props();
</script>

{#if actionMessage}
	<p class="setup-state success">{actionMessage}</p>
{/if}
{#if error}
	<p class="inline-error">{error}</p>
{/if}
