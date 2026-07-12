import { describe, expect, it } from 'vitest'
import {
  canExtractMailAttachmentText,
  extractionStatusLabel
} from './mailAttachmentTextExtractionPresentation'

describe('mail attachment text extraction presentation', () => {
  it('offers local extraction only after a clean scan verdict', () => {
    expect(canExtractMailAttachmentText({ scanStatus: 'clean' })).toBe(true)
    expect(canExtractMailAttachmentText({ scanStatus: 'not_scanned' })).toBe(false)
    expect(canExtractMailAttachmentText({ scanStatus: 'suspicious' })).toBe(false)
    expect(canExtractMailAttachmentText({ scanStatus: 'malicious' })).toBe(false)
  })

  it('keeps the server extraction outcomes distinct for the UI', () => {
    expect(extractionStatusLabel('completed')).toBe('ready')
    expect(extractionStatusLabel('unsupported')).toBe('unsupported')
  })
})
