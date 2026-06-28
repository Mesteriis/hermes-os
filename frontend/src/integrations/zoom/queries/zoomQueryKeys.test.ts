import { describe, expect, it } from 'vitest'
import { zoomQueryKeys } from './zoomQueryKeys'

describe('zoom query keys', () => {
  it('keeps every key under the integration zoom namespace', () => {
    for (const queryKey of Object.values(zoomQueryKeys)) {
      expect(queryKey[0]).toBe('integrations')
      expect(queryKey[1]).toBe('zoom')
    }
  })

  it('declares a dedicated recording import audit key', () => {
    expect(zoomQueryKeys.recordingImports).toEqual(['integrations', 'zoom', 'recording-imports'])
  })
})
