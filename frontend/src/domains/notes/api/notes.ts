import { ApiClient } from '../../../platform/api/ApiClient'
import type { NoteItem } from '../types/notes'

export async function fetchNotes(): Promise<{ items: NoteItem[] }> {
  return ApiClient.instance.get<{ items: NoteItem[] }>(
    '/api/v1/notes',
    'Notes request failed'
  )
}
