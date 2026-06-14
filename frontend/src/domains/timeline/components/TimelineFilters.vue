<script setup lang="ts">
import type { TimelineFilters as TimelineFiltersType } from '../types/timeline'

interface Props {
	filters: TimelineFiltersType
}

interface Emits {
	(e: 'toggleFilter', kind: keyof TimelineFiltersType): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const filterItems: Array<keyof TimelineFiltersType> = ['Messages', 'Documents', 'Tasks', 'Calendar', 'Notes', 'Decisions']
</script>

<template>
	<section class="panel info-card">
		<h2>Timeline Filters</h2>
		<label v-for="item in filterItems" :key="item" class="mini-check">
			<input
				type="checkbox"
				:checked="filters[item]"
				@change="emit('toggleFilter', item)"
			/>
			{{ item }}
		</label>
	</section>
</template>
