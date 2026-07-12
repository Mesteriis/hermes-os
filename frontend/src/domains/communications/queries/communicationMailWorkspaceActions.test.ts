import { describe, expect, it, vi } from 'vitest'
import {
  mailActionGroups,
  selectMailWorkspaceAction
} from './communicationMailWorkspaceActions'

describe('selectMailWorkspaceAction', () => {
  it('requeues a failed AI state through the page surface', async () => {
    const handleRetryAi = vi.fn().mockResolvedValue(undefined)
    const notifyMailActionError = vi.fn()
    const pageSurface = {
      store: { isMailActionRunning: false },
      handleRetryAi,
      notifyMailActionError
    }

    await selectMailWorkspaceAction(pageSurface as never, 'update-ai-state')

    expect(handleRetryAi).toHaveBeenCalledTimes(1)
    expect(notifyMailActionError).not.toHaveBeenCalled()
  })

  it('routes the not-spam action through the page surface', async () => {
    const handleMarkMessageNotSpam = vi.fn().mockResolvedValue(undefined)
    const notifyMailActionError = vi.fn()
    const pageSurface = {
      store: { isMailActionRunning: false },
      handleMarkMessageNotSpam,
      notifyMailActionError
    }

    await selectMailWorkspaceAction(pageSurface as never, 'mark-not-spam')

    expect(handleMarkMessageNotSpam).toHaveBeenCalledTimes(1)
    expect(notifyMailActionError).not.toHaveBeenCalled()
  })

  it('routes the star action and reflects the local starred state', async () => {
    const handleToggleStar = vi.fn().mockResolvedValue(undefined)
    const notifyMailActionError = vi.fn()
    const pageSurface = {
      store: { isMailActionRunning: false },
      handleToggleStar,
      notifyMailActionError
    }

    await selectMailWorkspaceAction(pageSurface as never, 'star')

    expect(handleToggleStar).toHaveBeenCalledTimes(1)
    const labels = (starred: boolean) =>
      mailActionGroups({
        importance_score: 0,
        message_metadata: { starred }
      } as never)
        .find((group) => group.id === 'organization')
        ?.actions.map((action) => action.label)
    expect(labels(false)).toContain('Star message')
    expect(labels(true)).toContain('Unstar message')
  })

  it('labels provider flag actions as local-only when the account cannot mutate flags', () => {
    const organizationActions = mailActionGroups({
      importance_score: 0,
      message_metadata: { starred: false }
    } as never, { providerFlagMutationAvailable: false })
      .find((group) => group.id === 'organization')
      ?.actions

    expect(organizationActions?.find((action) => action.id === 'important')?.description)
      .toContain('local')
    expect(organizationActions?.find((action) => action.id === 'star')?.description)
      .toMatch(/provider sync is unavailable/i)
  })

  it('shows the inverse spam action only for a spam workflow message', () => {
    const stateActionIds = (workflowState: string) =>
      mailActionGroups({ workflow_state: workflowState } as never)
        .find((group) => group.id === 'state')
        ?.actions.map((action) => action.id)

    expect(stateActionIds('spam')).toContain('mark-not-spam')
    expect(stateActionIds('spam')).not.toContain('mark-spam')
    expect(stateActionIds('new')).toContain('mark-spam')
    expect(stateActionIds('new')).not.toContain('mark-not-spam')
  })
})
