import type { ZoomRecordingRef } from '../types/zoom'

type UnknownRecord = Record<string, unknown>

export function extractZoomRecordingRefs(metadata: UnknownRecord | null | undefined): ZoomRecordingRef[] {
  const recordingRefs = metadata?.recording_refs
  if (!Array.isArray(recordingRefs)) return []

  return recordingRefs.filter(isZoomRecordingRef)
}

export function formatZoomTranscriptProvenance(provenance: unknown): string {
  if (!isUnknownRecord(provenance)) return '—'

  const normalized = sortObject(provenance)
  const entries = Object.entries(normalized)
  if (entries.length === 0) return '—'

  return JSON.stringify(normalized, null, 2)
}

function isZoomRecordingRef(value: unknown): value is ZoomRecordingRef {
  return isUnknownRecord(value) && typeof value.recording_id === 'string' && value.recording_id.trim().length > 0
}

function isUnknownRecord(value: unknown): value is UnknownRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function sortObject(value: UnknownRecord): UnknownRecord {
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => {
        if (Array.isArray(entry)) {
          return [key, entry.map((item) => (isUnknownRecord(item) ? sortObject(item) : item))]
        }
        if (isUnknownRecord(entry)) {
          return [key, sortObject(entry)]
        }
        return [key, entry]
      })
  )
}
