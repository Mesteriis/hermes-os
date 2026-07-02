<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { MediaTone } from './Media.types'

const props = withDefaults(defineProps<{
	name: string
	mimeType?: string
	size?: string
	description?: string
	icon?: string
	tone?: MediaTone
	class?: string
}>(), {
	icon: 'tabler:paperclip',
	tone: 'neutral'
})

const classes = computed(() => [
	'hermes-attachment-preview',
	`hermes-attachment-preview--${props.tone}`,
	props.class
])
const meta = computed(() => [props.mimeType, props.size].filter(Boolean).join(' · '))
</script>

<template>
	<article :class="classes">
		<div class="hermes-attachment-preview__icon" aria-hidden="true">
			<Icon :icon="icon" size="1.25rem" />
		</div>
		<div class="hermes-attachment-preview__body">
			<h3 class="hermes-attachment-preview__name">{{ name }}</h3>
			<p v-if="description" class="hermes-attachment-preview__description">{{ description }}</p>
			<span v-if="meta" class="hermes-attachment-preview__meta">{{ meta }}</span>
		</div>
		<div v-if="$slots.action" class="hermes-attachment-preview__action">
			<slot name="action" />
		</div>
	</article>
</template>
