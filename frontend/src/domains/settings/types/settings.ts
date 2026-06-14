export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json'

export interface ApplicationSetting {
  setting_key: string
  category: string
  value_kind: SettingValueKind
  value: boolean | number | string | Record<string, unknown> | unknown[]
  label: string
  description: string
  metadata: Record<string, unknown>
  is_editable: boolean
  updated_by_actor_id: string | null
  created_at: string
  updated_at: string
}

export interface ApplicationSettingsResponse {
  items: ApplicationSetting[]
}

export interface ProviderAccount {
  account_id: string
  provider_kind: 'gmail' | 'icloud' | 'imap' | string
  email: string
  label: string
  is_active: boolean
  is_authenticated: boolean
  last_sync_at: string | null
  created_at: string
}

export interface ProviderAccountListResponse {
  items: ProviderAccount[]
}

export interface CalendarAccount {
  id: string
  provider_kind: string
  email: string
  label: string
  is_active: boolean
  calendar_ids: string[]
}

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout'
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar'
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale'
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme'
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state'
