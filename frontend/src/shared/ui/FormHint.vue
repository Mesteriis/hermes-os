<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	class?: string
}>(), {})

const classes = computed(() => ['hermes-form-hint', props.class])
</script>

<template>
	<p :id="id" :class="classes">
		<slot />
	</p>
</template>
