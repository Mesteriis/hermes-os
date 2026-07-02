<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	class?: string
}>(), {
	as: 'div'
})

const classes = computed(() => ['hermes-field', props.class])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
