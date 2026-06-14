import { useQuery } from '@tanstack/vue-query'
import { fetchNotes } from '../api/notes'

export function useNotesQuery() {
  return useQuery({
    queryKey: ['notes'],
    queryFn: () => fetchNotes()
  })
}
