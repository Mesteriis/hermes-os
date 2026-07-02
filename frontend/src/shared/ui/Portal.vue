<script setup lang="ts">
const props = withDefaults(defineProps<{
	to?: string
	disabled?: boolean
}>(), {
	to: 'body',
	disabled: false
})
</script>

<template>
	<Teleport :to="props.to" :disabled="props.disabled">
		<slot />
	</Teleport>
</template>
