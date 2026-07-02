<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	class?: string
}>(), {})

const classes = computed(() => ['hermes-form-error', props.class])
</script>

<template>
	<p :id="id" :class="classes" role="alert">
		<slot />
	</p>
</template>
