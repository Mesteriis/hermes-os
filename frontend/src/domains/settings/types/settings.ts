export type {
  ApplicationSetting,
  ApplicationSettingsResponse,
  ApplicationSettingValue,
  SettingValueKind
} from '../../../platform/settings/applicationSettingsClient'

export {
  FRONTEND_LAYOUT_SETTING_KEY,
  FRONTEND_SIDEBAR_SETTING_KEY,
  FRONTEND_LOCALE_SETTING_KEY,
  FRONTEND_THEME_SETTING_KEY,
  FRONTEND_UI_STATE_SETTING_KEY
} from '../../../platform/settings/applicationSettingsClient'

export interface ProviderAccount {
  account_id: string
  provider_kind: 'gmail' | 'icloud' | 'imap' | string
  display_name: string
  external_account_id: string
  config: Record<string, unknown>
  created_at: string
  updated_at: string
  email?: string | null
  label?: string | null
  is_active?: boolean
  is_authenticated?: boolean
  last_sync_at?: string | null
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
