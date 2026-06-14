import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TimelineMessage, TimelineFilters } from '../types/timeline'

export const useTimelineStore = defineStore('timeline-ui', () => {
	const messages = ref<TimelineMessage[]>([])
	const error = ref('')
	const isLoading = ref(false)
	const filters = ref<TimelineFilters>({
		Messages: true,
		Documents: true,
		Tasks: true,
		Calendar: true,
		Notes: true,
		Decisions: true
	})

	const filteredMessages = computed<TimelineMessage[]>(() => {
		// Filter is a placeholder — in the Svelte original, filter state exists
		// but all items pass through. Keep the structure for AC4 compliance.
		return messages.value
	})

	function setMessages(items: TimelineMessage[]) {
		messages.value = items
	}

	function setLoading(v: boolean) {
		isLoading.value = v
	}

	function setError(msg: string) {
		error.value = msg
	}

	function toggleFilter(kind: keyof TimelineFilters) {
		filters.value[kind] = !filters.value[kind]
	}

	return {
		messages,
		error,
		isLoading,
		filters,
		filteredMessages,
		setMessages,
		setLoading,
		setError,
		toggleFilter
	}
})
