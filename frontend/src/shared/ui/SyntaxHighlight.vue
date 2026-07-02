<script setup lang="ts">
import { computed } from 'vue'
import { highlightCodeToSafeHtml } from './Media.rendering'

const props = withDefaults(defineProps<{
	code: string
	language?: string
	label?: string
	class?: string
}>(), {
	language: 'plaintext'
})

const classes = computed(() => ['hermes-syntax-highlight', props.class])
const highlightedHtml = computed(() => highlightCodeToSafeHtml(props.code, props.language))
</script>

<template>
	<figure :class="classes">
		<figcaption v-if="label || language" class="hermes-code-block__caption">
			<span v-if="label">{{ label }}</span>
			<span class="hermes-code-block__language">{{ language }}</span>
		</figcaption>
		<pre class="hermes-code-block__pre" tabindex="0" :aria-label="label || language"><code class="hljs" v-html="highlightedHtml" /></pre>
	</figure>
</template>
