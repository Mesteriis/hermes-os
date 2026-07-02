<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger'
	icon?: string
	class?: string
}>(), {
	tone: 'neutral'
})

const classes = computed(() => ['hermes-tag', `hermes-tag--${props.tone}`, props.class])
</script>

<template>
	<span :class="classes">
		<Icon v-if="icon" :icon="icon" size="1em" />
		<span class="hermes-tag__label"><slot /></span>
	</span>
</template>
