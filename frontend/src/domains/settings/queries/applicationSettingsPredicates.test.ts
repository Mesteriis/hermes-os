import { describe, expect, it } from 'vitest'
import { isPublicApplicationSetting, settingHasChanged } from './applicationSettingsPredicates'
import type { ApplicationSetting } from '../types/settings'

describe('application settings predicates', () => {
  it('hides AI and communications settings from the public registry', () => {
    expect(isPublicApplicationSetting(setting('general'))).toBe(true)
    expect(isPublicApplicationSetting(setting('ai'))).toBe(false)
    expect(isPublicApplicationSetting(setting('communications'))).toBe(false)
    expect(isPublicApplicationSetting({ ...setting('general'), setting_key: 'ai.hidden' })).toBe(false)
  })

  it('detects only changed drafts', () => {
    const value = setting('general')
    expect(settingHasChanged(value, {})).toBe(false)
    expect(settingHasChanged(value, { 'general.setting': 'value' })).toBe(false)
    expect(settingHasChanged(value, { 'general.setting': 'changed' })).toBe(true)
  })
})

function setting(category: string): ApplicationSetting {
  return {
    setting_key: category === 'general' ? 'general.setting' : `${category}.setting`,
    category,
    label: 'Setting',
    description: 'Description',
    value_kind: 'string',
    value: 'value',
    metadata: {},
    is_editable: true,
    updated_by_actor_id: null,
    created_at: '2026-07-21T00:00:00Z',
    updated_at: '2026-07-21T00:00:00Z'
  }
}
