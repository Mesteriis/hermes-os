<script lang="ts">
	import { aiSettingsActionMessage, aiSettingsError } from '$lib/stores/aiSettings';
</script>

{#if $aiSettingsActionMessage}
	<p class="setup-state success">{$aiSettingsActionMessage}</p>
{/if}
{#if $aiSettingsError}
	<p class="inline-error">{$aiSettingsError}</p>
{/if}
