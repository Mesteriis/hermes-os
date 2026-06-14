import { useQuery } from '@tanstack/vue-query'
import { fetchGraphSummary, fetchContradictions } from '../api/knowledge'

export function useGraphSummaryQuery() {
	return useQuery({
		queryKey: ['graph-summary'],
		queryFn: async () => {
			const summary = await fetchGraphSummary()
			return summary
		}
	})
}

export function useContradictionsQuery(limit = 50) {
	return useQuery({
		queryKey: ['contradictions', limit],
		queryFn: async () => {
			const response = await fetchContradictions(limit)
			return response.items
		}
	})
}
