import { describe, expect, it, vi } from 'vitest'
import { updateMailContentEgress } from './communicationsContentEgressActions'

describe('communications content egress actions', () => {
  it('updates only the requested content permission', async () => {
    const dependencies = dependenciesFor()

    await updateMailContentEgress('account-1', 'body', true, dependencies)

    expect(dependencies.updateContentEgressSettings).toHaveBeenCalledWith({
      accountId: 'account-1',
      settings: { body: true }
    })
    expect(dependencies.setActionMessage).toHaveBeenCalledWith('Mail content access preference saved')
  })

  it('does not mutate without a selected account', async () => {
    const dependencies = dependenciesFor()

    await updateMailContentEgress(null, 'attachments', true, dependencies)

    expect(dependencies.updateContentEgressSettings).not.toHaveBeenCalled()
  })
})

function dependenciesFor() {
  return {
    clearMessages: vi.fn(),
    setActionMessage: vi.fn(),
    setError: vi.fn(),
    updateContentEgressSettings: vi.fn().mockResolvedValue({})
  }
}
