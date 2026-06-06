<script lang="ts">
	import '$lib/styles/tokens.css';
	import '$lib/styles/app.css';

	let { children } = $props();
</script>

<div class="viewport-guard" role="alert" aria-live="polite">
	<strong>Minimum window size is 800 x 600</strong>
	<span>Increase the Hermes Hub window size to continue.</span>
</div>

{@render children()}
