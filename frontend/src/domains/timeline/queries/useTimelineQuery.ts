import { useQuery } from '@tanstack/vue-query'
import { fetchCommunicationMessages } from '../api/timeline'

export function useTimelineMessagesQuery() {
	return useQuery({
		queryKey: ['timeline-messages'],
		queryFn: () => fetchCommunicationMessages(500),
		refetchOnMount: 'always' as const,
		staleTime: 30_000
	})
}
