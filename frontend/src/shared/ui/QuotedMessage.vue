<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	author?: string
	body?: string
	meta?: string
	class?: string
}>(), {})

const classes = computed(() => ['hermes-quoted-message', props.class])
</script>

<template>
	<blockquote :class="classes">
		<header v-if="author || meta" class="hermes-quoted-message__header">
			<strong v-if="author">{{ author }}</strong>
			<span v-if="meta">{{ meta }}</span>
		</header>
		<p v-if="body" class="hermes-quoted-message__body">{{ body }}</p>
		<div v-else class="hermes-quoted-message__body">
			<slot />
		</div>
	</blockquote>
</template>
