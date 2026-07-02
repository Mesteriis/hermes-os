<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { MediaTrack } from './Media.types'

const props = withDefaults(defineProps<{
	src?: string
	poster?: string
	title?: string
	description?: string
	tracks?: MediaTrack[]
	preload?: 'none' | 'metadata' | 'auto'
	fallbackLabel?: string
	class?: string
}>(), {
	tracks: () => [],
	preload: 'metadata',
	fallbackLabel: 'Video unavailable'
})

const classes = computed(() => ['hermes-video-player', props.class])
const mediaLabel = computed(() => props.title || props.fallbackLabel)
</script>

<template>
	<section :class="classes" :aria-label="mediaLabel">
		<header v-if="title || description" class="hermes-media-header">
			<h3 v-if="title" class="hermes-media-title">{{ title }}</h3>
			<p v-if="description" class="hermes-media-description">{{ description }}</p>
		</header>
		<video
			v-if="src"
			class="hermes-video-player__asset"
			:src="src"
			:poster="poster"
			:preload="preload"
			:aria-label="mediaLabel"
			controls
		>
			<track
				v-for="track in tracks"
				:key="track.src"
				:src="track.src"
				:kind="track.kind"
				:label="track.label"
				:srclang="track.srclang"
				:default="track.default"
			/>
		</video>
		<div v-else class="hermes-media-empty" role="status">
			<Icon icon="tabler:video-off" size="1.25rem" aria-hidden="true" />
			<span>{{ fallbackLabel }}</span>
		</div>
	</section>
</template>
