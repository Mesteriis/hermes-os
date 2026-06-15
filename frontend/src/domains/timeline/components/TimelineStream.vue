<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import type { TimelineMessage } from '../types/timeline'

interface Props {
	messages: TimelineMessage[]
}

const props = defineProps<Props>()

const parentRef = ref<HTMLElement | null>(null)

const virtualizer = useVirtualizer(computed(() => ({
	count: props.messages.length,
	getScrollElement: () => parentRef.value,
	estimateSize: () => 72,
	overscan: 10
})))

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
	<section class="panel feed-panel large-timeline" ref="parentRef">
		<header class="panel-title-row">
			<h2>Today</h2>
			<button type="button" class="ghost-button" disabled>All Events</button>
		</header>

		<div :style="{ height: `${totalSize}px`, position: 'relative' }">
			<article
				v-for="virtualRow in virtualItems"
				:key="String(virtualRow.key)"
				class="timeline-event-row"
				:style="{
					position: 'absolute',
					top: 0,
					left: 0,
					width: '100%',
					transform: `translateY(${virtualRow.start}px)`
				}"
			>
				<span class="rail-dot"></span>
				<span class="round-icon blue"><Icon icon="tabler:message" width="20" height="20" /></span>
				<div>
					<strong>{{ messages[virtualRow.index].sender_display_name || messages[virtualRow.index].sender || 'Unknown' }}</strong>
					<p>{{ messages[virtualRow.index].subject || messages[virtualRow.index].body_text_preview }}</p>
					<time>{{ messages[virtualRow.index].occurred_at || messages[virtualRow.index].projected_at }}</time>
				</div>
			</article>
		</div>
	</section>
</template>

<style scoped>
.large-timeline {
	max-height: 100%;
	overflow-y: auto;
	padding: 0 0 12px;
}

.timeline-event-row {
	display: grid;
	grid-template-columns: 64px 18px 40px 1fr;
	gap: 10px;
	min-height: 72px;
	border-bottom: 1px solid rgba(102, 189, 180, 0.08);
	padding: 12px 18px;
}

.timeline-event-row time {
	color: var(--hh-color-accent);
	font-size: 12px;
}

.rail-dot {
	width: 8px;
	height: 8px;
	margin-top: 8px;
	border-radius: var(--hh-radius-round);
	background: var(--hh-color-accent);
	box-shadow: 0 0 12px rgba(45, 240, 206, 0.85);
}

.timeline-event-row strong {
	color: var(--hh-color-text-bright);
	font-size: 13px;
}

.timeline-event-row p {
	margin-top: 5px;
	color: var(--hh-color-text-muted);
	font-size: 12px;
}
</style>
