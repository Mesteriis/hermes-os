import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  fetchDocumentProcessingJobs,
  retryDocumentProcessingJob,
} from '../api/documents'
import type { DocumentProcessingRetryRequest } from '../types/documents'

export function useDocumentProcessingJobsQuery(limit = 50) {
  return useQuery({
    queryKey: ['document-processing-jobs', limit],
    queryFn: () => fetchDocumentProcessingJobs(limit)
  })
}

export function useRetryDocumentProcessingJobMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ jobId, request }: { jobId: string; request: DocumentProcessingRetryRequest }) =>
      retryDocumentProcessingJob(jobId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document-processing-jobs'] })
    },
  })
}
