import { describe, expect, it } from 'vitest'
import {
  categoryLabel,
  settingAllowedValues,
  settingControlType,
  settingDraftValue,
  settingMetadataFlag,
  settingMetadataText
} from './applicationSettingsPresentation'
import type { ApplicationSetting } from '../types/settings'

describe('application settings presentation', () => {
  it('builds controls and metadata values from setting contracts', () => {
    const setting: ApplicationSetting = {
      setting_key: 'general.mode',
      category: 'general',
      label: 'Mode',
      description: 'Mode',
      value_kind: 'string',
      value: 'safe',
      metadata: { allowed_values: ['safe', 'fast'], visible: true, help: 'Choose a mode' },
      is_editable: true,
      updated_by_actor_id: null,
      created_at: '2026-07-21T00:00:00Z',
      updated_at: '2026-07-21T00:00:00Z'
    }

    expect(settingControlType(setting)).toBe('select')
    expect(settingAllowedValues(setting)).toEqual(['safe', 'fast'])
    expect(settingDraftValue(setting, {})).toBe('safe')
    expect(settingDraftValue(setting, { 'general.mode': 'fast' })).toBe('fast')
    expect(settingMetadataFlag(setting, 'visible')).toBe(true)
    expect(settingMetadataText(setting, 'help')).toBe('Choose a mode')
    expect(categoryLabel('frontend')).toBe('Interface')
  })
})
