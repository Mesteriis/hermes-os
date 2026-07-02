<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	src?: string
	title?: string
	description?: string
	preload?: 'none' | 'metadata' | 'auto'
	fallbackLabel?: string
	class?: string
}>(), {
	preload: 'metadata',
	fallbackLabel: 'Audio unavailable'
})

const classes = computed(() => ['hermes-audio-player', props.class])
const mediaLabel = computed(() => props.title || props.fallbackLabel)
</script>

<template>
	<section :class="classes" :aria-label="mediaLabel">
		<div class="hermes-audio-player__copy">
			<Icon icon="tabler:wave-sine" size="1.25rem" aria-hidden="true" />
			<div>
				<h3 v-if="title" class="hermes-media-title">{{ title }}</h3>
				<p v-if="description" class="hermes-media-description">{{ description }}</p>
			</div>
		</div>
		<audio v-if="src" class="hermes-audio-player__asset" :src="src" :preload="preload" :aria-label="mediaLabel" controls />
		<div v-else class="hermes-media-empty" role="status">
			<Icon icon="tabler:volume-off" size="1.25rem" aria-hidden="true" />
			<span>{{ fallbackLabel }}</span>
		</div>
	</section>
</template>
