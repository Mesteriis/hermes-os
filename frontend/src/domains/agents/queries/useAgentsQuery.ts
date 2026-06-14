import { useQuery } from '@tanstack/vue-query'
import { fetchAiAgents, fetchAiRuns, fetchAiStatus, fetchOwnerPersona } from '../api/agents'

export function useAiWorkspaceQuery() {
	return useQuery({
		queryKey: ['ai-workspace'],
		queryFn: async () => {
			const [agentResponse, runResponse, ownerResponse] = await Promise.all([
				fetchAiAgents(),
				fetchAiRuns(25),
				fetchOwnerPersona()
			])
			const agents = agentResponse.items
			const runs = runResponse.items
			const ownerPersona = ownerResponse.owner_persona
			let aiStatus: Awaited<ReturnType<typeof fetchAiStatus>> | null = null
			let error = ''
			try {
				aiStatus = await fetchAiStatus()
			} catch (statusError) {
				error = statusError instanceof Error ? statusError.message : 'Unknown AI status error'
			}
			return { agents, runs, status: aiStatus, ownerPersona, error }
		},
		refetchOnMount: 'always' as const,
		staleTime: 30_000
	})
}

export function useAiRunsQuery() {
	return useQuery({
		queryKey: ['ai-runs'],
		queryFn: async () => {
			const response = await fetchAiRuns(25)
			return { runs: response.items, error: '' }
		},
		staleTime: 10_000
	})
}
