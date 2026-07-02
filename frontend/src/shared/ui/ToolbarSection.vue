<script setup lang="ts">
import { computed, useId } from 'vue'

type ToolbarSectionOrientation = 'horizontal' | 'vertical'

const props = withDefaults(defineProps<{
	orientation?: ToolbarSectionOrientation
	class?: string
}>(), {
	orientation: 'horizontal'
})

const classes = computed(() => [
	'hermes-toolbar-section',
	`hermes-toolbar-section--${props.orientation}`,
	props.class
])

const labelId = `hermes-toolbar-section-label-${useId()}`
</script>

<template>
	<section
		:class="classes"
		role="group"
		:aria-labelledby="$slots.label ? labelId : undefined"
	>
		<span
			v-if="$slots.label"
			:id="labelId"
			class="hermes-toolbar-section-label"
		>
			<slot name="label" />
		</span>
		<div class="hermes-toolbar-section-content">
			<slot />
		</div>
	</section>
</template>
