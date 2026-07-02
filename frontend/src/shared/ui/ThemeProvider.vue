<script setup lang="ts">
import { computed } from 'vue'
import type { UiThemeName } from './theme'

const props = withDefaults(defineProps<{
	as?: string
	theme?: UiThemeName
	class?: string
}>(), {
	as: 'div',
	theme: 'hermes'
})

const classes = computed(() => ['hermes-theme-provider', props.class])
</script>

<template>
	<component :is="as" :data-ui-theme="theme" :class="classes">
		<slot />
	</component>
</template>
