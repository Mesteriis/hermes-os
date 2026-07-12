import { describe, expect, it } from 'vitest'
import { mailImportKindForFile } from './mailImport'

describe('mail import file validation', () => {
  it('accepts bounded EML and MBOX files', () => {
    expect(mailImportKindForFile({ name: 'message.eml', size: 512 })).toBe('eml')
    expect(mailImportKindForFile({ name: 'Archive.MBOX', size: 20 * 1024 * 1024 })).toBe('mbox')
  })

  it('rejects empty, oversized, and unsupported files before they are read', () => {
    expect(() => mailImportKindForFile({ name: 'message.eml', size: 0 })).toThrow('empty')
    expect(() => mailImportKindForFile({ name: 'archive.mbox', size: 20 * 1024 * 1024 + 1 })).toThrow('20 MiB')
    expect(() => mailImportKindForFile({ name: 'message.pdf', size: 512 })).toThrow('.eml or .mbox')
  })
})
