import { describe, expect, it } from 'vitest'
import { mailReadSyncPresentation } from './mailReadSyncPresentation'

describe('mailReadSyncPresentation', () => {
  it('keeps synchronized messages quiet and explains pending or failed provider state', () => {
    expect(mailReadSyncPresentation('synced')).toBeNull()
    expect(mailReadSyncPresentation('queued')).toMatchObject({ tone: 'info' })
    expect(mailReadSyncPresentation('retrying')).toMatchObject({ tone: 'warning' })
    expect(mailReadSyncPresentation('failed')).toEqual({
      icon: 'tabler:cloud-off',
      label: 'Provider sync failed; local state is preserved',
      tone: 'danger'
    })
  })
})
