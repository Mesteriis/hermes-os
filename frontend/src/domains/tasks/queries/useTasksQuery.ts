import { useQuery } from '@tanstack/vue-query'
import { fetchTaskCandidates, fetchTaskRecords } from '../api/tasks'
import type { TaskCandidate, Task } from '../types/task'

export function useTaskCandidatesQuery() {
  return useQuery<TaskCandidate[]>({
    queryKey: ['task-candidates'],
    queryFn: async () => {
      const response = await fetchTaskCandidates(50)
      return response.items
    }
  })
}

export function useTasksQuery() {
  return useQuery<Task[]>({
    queryKey: ['tasks'],
    queryFn: async () => {
      const response = await fetchTaskRecords({ limit: 50 })
      return response.items
    }
  })
}
