<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	class?: string
}>(), {})

const classes = computed(() => ['hermes-kbd', props.class])
</script>

<template>
	<kbd :class="classes">
		<slot />
	</kbd>
</template>
