import { useQuery } from '@tanstack/vue-query'
import { fetchDocumentProcessingJobs } from '../api/documents'

export function useDocumentProcessingJobsQuery(limit = 50) {
  return useQuery({
    queryKey: ['document-processing-jobs', limit],
    queryFn: () => fetchDocumentProcessingJobs(limit)
  })
}
