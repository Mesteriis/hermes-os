<script setup lang="ts">
type SurfaceTone = 'panel' | 'raised' | 'deep'

withDefaults(
	defineProps<{
		as?: string
		tone?: SurfaceTone
	}>(),
	{
		as: 'section',
		tone: 'panel'
	}
)
</script>

<template>
	<component :is="as" class="hh-surface" :class="`hh-surface--${tone}`">
		<slot />
	</component>
</template>
