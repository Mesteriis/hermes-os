import { describe, expect, it } from 'vitest'
import {
  signalControlAvailability,
  signalTargetIcon,
} from './signalHubRoutePresentation'

describe('signal hub route presentation', () => {
  it('maps route target kinds to their icons', () => {
    expect(signalTargetIcon('projection')).toBe('tabler:chart-dots')
    expect(signalTargetIcon('consumer')).toBe('tabler:route')
  })

  it('derives source-control button availability from capability and state', () => {
    const row = {
      source: { supports_pause: true, supports_mute: true },
      state: 'paused' as const,
    }

    expect(signalControlAvailability(row, false)).toEqual({
      pauseDisabled: true,
      resumeDisabled: false,
      muteDisabled: false,
      unmuteDisabled: true,
      disableDisabled: false,
      enableDisabled: true,
    })
    expect(signalControlAvailability(row, true).resumeDisabled).toBe(true)
  })
})
