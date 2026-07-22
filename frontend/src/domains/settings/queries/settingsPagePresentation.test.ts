import { describe, expect, it } from 'vitest'
import { buildSettingsOverviewCards, buildSettingsTreeGroups } from './settingsPagePresentation'

describe('settings page presentation', () => {
  it('builds navigation metadata from surface counts and loading state', () => {
    const groups = buildSettingsTreeGroups({
      integrationCount: 2,
      communicationsAccountCount: 1,
      applicationSettingsCount: 3,
      backgroundJobCount: 4,
      backgroundJobsLoading: true,
      traceSpanCount: 5,
      traceLogsLoading: false,
      maintenanceTotalSizeLabel: '12 MB',
      maintenanceLoading: false,
      aiProviderCount: 1,
      signalSourceCount: 6,
      signalHubLoading: false
    }, (key) => `translated:${key}`)

    expect(groups[0]?.items.find((item) => item.id === 'accounts')?.meta).toBe('2')
    expect(groups[0]?.items.find((item) => item.id === 'background-jobs')?.meta).toBe('translated:Loading...')
    expect(groups[1]?.items.find((item) => item.id === 'signal-hub')?.meta).toBe('6')
  })

  it('builds overview card tone and loading values', () => {
    const cards = buildSettingsOverviewCards({
      realtimeStatusLabel: 'Connected',
      realtimeStatusTone: 'success',
      realtimeHasError: false,
      integrationCount: 0,
      applicationSettingsCount: 2,
      applicationSettingsLoading: true,
      aiProviderCount: 1,
      aiLoading: false
    }, (key) => `translated:${key}`)

    expect(cards.find((card) => card.id === 'realtime')).toMatchObject({ tone: 'success' })
    expect(cards.find((card) => card.id === 'sources')).toMatchObject({ tone: 'neutral', value: '0' })
    expect(cards.find((card) => card.id === 'registry')).toMatchObject({ value: 'translated:Loading...' })
  })
})
