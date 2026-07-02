<script setup lang="ts">
import { computed } from 'vue'
import { sanitizeHtml } from './Media.rendering'

const props = withDefaults(defineProps<{
	content?: string
	format?: 'html' | 'text'
	sanitized?: boolean
	title?: string
	unsafeLabel?: string
	emptyLabel?: string
	class?: string
}>(), {
	content: '',
	format: 'text',
	sanitized: false,
	unsafeLabel: 'HTML preview requires sanitized content',
	emptyLabel: 'No preview content'
})

const classes = computed(() => [
	'hermes-html-preview',
	`hermes-html-preview--${props.format}`,
	{
		'hermes-html-preview--blocked': props.format === 'html' && !props.sanitized
	},
	props.class
])
const hasContent = computed(() => props.content.trim().length > 0)
const canRenderHtml = computed(() => props.format === 'html' && props.sanitized && hasContent.value)
const safeHtml = computed(() => {
	if (!canRenderHtml.value) {
		return ''
	}
	return sanitizeHtml(props.content)
})
</script>

<template>
	<article :class="classes">
		<h3 v-if="title" class="hermes-media-title">{{ title }}</h3>
		<div v-if="canRenderHtml" class="hermes-html-preview__content" v-html="safeHtml" />
		<pre v-else-if="hasContent" class="hermes-html-preview__text">{{ content }}</pre>
		<p v-else class="hermes-media-empty">{{ emptyLabel }}</p>
		<p v-if="format === 'html' && hasContent && !sanitized" class="hermes-html-preview__safety">
			{{ unsafeLabel }}
		</p>
	</article>
</template>
