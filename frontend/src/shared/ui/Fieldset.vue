<script setup lang="ts">
import { computed, useId } from 'vue'

const props = withDefaults(defineProps<{
	disabled?: boolean
	class?: string
}>(), {
	disabled: false
})

const classes = computed(() => [
	'hermes-fieldset',
	props.disabled && 'hermes-fieldset--disabled',
	props.class
])

const descriptionId = `hermes-fieldset-description-${useId()}`
</script>

<template>
	<fieldset
		:class="classes"
		:disabled="disabled"
		:aria-describedby="$slots.description ? descriptionId : undefined"
	>
		<legend v-if="$slots.legend" class="hermes-fieldset-legend">
			<slot name="legend" />
		</legend>
		<p
			v-if="$slots.description"
			:id="descriptionId"
			class="hermes-fieldset-description"
		>
			<slot name="description" />
		</p>
		<div class="hermes-fieldset-content">
			<slot />
		</div>
	</fieldset>
</template>
