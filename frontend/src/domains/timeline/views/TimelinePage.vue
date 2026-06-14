<script setup lang="ts">
import { watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import { useTimelineMessagesQuery } from '../queries/useTimelineQuery'
import { useTimelineStore } from '../stores/timeline'
import TimelineStream from '../components/TimelineStream.vue'
import TimelineFilters from '../components/TimelineFilters.vue'

const store = useTimelineStore()

const { data: messagesData, isLoading } = useTimelineMessagesQuery()

watch(messagesData, (val) => {
	if (val) {
		store.setMessages(val)
		store.setLoading(false)
	}
})

watch(isLoading, (val) => {
	store.setLoading(val)
})
</script>

<template>
	<section class="timeline-page">
		<div class="view-header">
			<div class="view-title-with-icon">
				<span class="hero-mark small"><Icon icon="tabler:timeline-event" width="28" height="28" /></span>
				<div>
					<h1>Timeline</h1>
					<p>Chronological activity across connected sources.</p>
				</div>
			</div>
		</div>
		<div class="timeline-layout">
			<TimelineStream :messages="store.filteredMessages" />
			<aside class="stacked-rail">
				<TimelineFilters :filters="store.filters" @toggle-filter="store.toggleFilter($event)" />
			</aside>
		</div>
	</section>
</template>

<style scoped>
.timeline-page {
	display: grid;
	grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
	grid-auto-flow: row;
	grid-auto-rows: min-content;
	align-content: start;
	gap: var(--hh-layout-gap);
	height: 100%;
	min-height: 0;
	overflow: hidden;
	padding-right: 0;
}

.timeline-page > * {
	grid-column: 1 / -1;
	min-width: 0;
}

.timeline-layout {
	--hh-zone-rows: 12;

	display: grid;
	grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
	grid-auto-flow: dense;
	grid-auto-rows: min-content;
	align-content: start;
	align-items: stretch;
	gap: var(--hh-layout-gap);
	width: 100%;
	min-width: 0;
	min-height: 0;
	max-height: 100%;
	overflow: hidden;
}
</style>
