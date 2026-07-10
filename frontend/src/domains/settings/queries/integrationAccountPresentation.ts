import type { ConnectionProviderId } from '../../../shared/stores/integrationConnectionWizard'
import type { CalendarAccount, ProviderAccount } from '../types/settings'

export interface IntegrationAccountRow {
  account: ProviderAccount
  displayName: string
  icon: string
  iconTone: string
  providerLabel: string
  flowLabel: string
  statusText: string
  statusClass: string
  isSelected: boolean
}

export interface IntegrationGroup {
  label: string
  items: IntegrationAccountRow[]
}

export interface SelectedIntegrationSummary {
  account: ProviderAccount
  displayName: string
  icon: string
  iconTone: string
  providerLabel: string
  flowLabel: string
  email: string
  statusText: string
  nextStepLabel: string
  selectedInspectorActionLabel: string
  canManageMail: boolean
  canOpenSetupAction: boolean
  canRecoverExpiredCredential: boolean
  accountEnabled: boolean
  accountToggleLabel: string
  accountToggleHelp: string
  labelDraft: string
  labelDirty: boolean
  labelSaving: boolean
  services: AccountServiceRow[]
}

export interface AccountServiceRow {
  id: 'mail' | 'calendar' | 'contacts' | 'messenger' | 'meetings'
  label: string
  icon: string
  enabled: boolean
  statusText: string
  detail: string
  canToggle: boolean
  canRunNow?: boolean
  runNowLabel?: string
  modeActionLabel?: string
  canRunModeAction?: boolean
  disabledReason?: string
  isBusy: boolean
}

const PROVIDER_LABELS: Record<string, string> = {
  gmail: 'Gmail',
  icloud: 'iCloud',
  imap: 'IMAP',
  telegram_user: 'Telegram User',
  telegram_bot: 'Telegram Bot',
  whatsapp_web: 'WhatsApp Web',
  whatsapp_business_cloud: 'WhatsApp Business Cloud',
  zoom_user: 'Zoom (OAuth/Live)',
  zoom_server_to_server: 'Zoom (Server-to-Server)',
  yandex_telemost_user: 'Yandex Telemost',
  zulip_bot: 'Zulip Bot',
}

export function isMailProvider(providerKind: string): boolean {
  return providerKind === 'gmail' || providerKind === 'icloud' || providerKind === 'imap'
}

export function supportsCalendar(providerKind: string): boolean {
  return providerKind === 'gmail' || providerKind === 'icloud'
}

export function isZoomProvider(providerKind: string): boolean {
  return providerKind === 'zoom_user' || providerKind === 'zoom_server_to_server'
}

export function isTelegramProvider(providerKind: string): boolean {
  return providerKind === 'telegram_user' || providerKind === 'telegram_bot'
}

export function isWhatsappProvider(providerKind: string): boolean {
  return providerKind === 'whatsapp_web' || providerKind === 'whatsapp_business_cloud'
}

export function isYandexTelemostProvider(providerKind: string): boolean {
  return providerKind === 'yandex_telemost_user'
}

export function isZulipProvider(providerKind: string): boolean {
  return providerKind === 'zulip_bot'
}

export function isExceptionOnlyProvider(providerKind: string): boolean {
  return providerKind === 'icloud' || providerKind === 'imap'
}

export function isExceptionRouteProvider(providerKind: string): boolean {
  return isZulipProvider(providerKind)
}

export function isManagedRuntimeProvider(providerKind: string): boolean {
  return (
    isZoomProvider(providerKind) ||
    isTelegramProvider(providerKind) ||
    isWhatsappProvider(providerKind) ||
    isYandexTelemostProvider(providerKind) ||
    isZulipProvider(providerKind)
  )
}

function accountConnectedServices(account: ProviderAccount): string[] {
  const raw = account.config?.connected_services
  if (!Array.isArray(raw)) return []
  return raw.filter((service): service is string => typeof service === 'string')
}

export function accountConfigBoolean(account: ProviderAccount, key: string): boolean | null {
  const value = account.config?.[key]
  return typeof value === 'boolean' ? value : null
}

export function accountConfigString(account: ProviderAccount, key: string): string | null {
  const value = account.config?.[key]
  return typeof value === 'string' ? value : null
}

export function accountSupportsContacts(account: ProviderAccount): boolean {
  return accountConnectedServices(account).includes('contacts')
}

export function accountContactsSyncDirection(
  account: ProviderAccount
): 'read_only' | 'bidirectional' {
  return accountConfigString(account, 'address_book_sync_direction') === 'bidirectional'
    ? 'bidirectional'
    : 'read_only'
}

export function accountContactsRemoteWriteEnabled(account: ProviderAccount): boolean {
  return accountConfigBoolean(account, 'address_book_remote_write_enabled') ?? false
}

function accountRequestedScopes(account: ProviderAccount): string[] {
  const raw = account.config?.requested_scopes
  if (!Array.isArray(raw)) return []
  return raw.filter((scope): scope is string => typeof scope === 'string')
}

export function accountHasGoogleContactsWriteScope(account: ProviderAccount): boolean {
  return accountRequestedScopes(account).includes('https://www.googleapis.com/auth/contacts')
}

export function providerLabel(providerKind: string): string {
  return PROVIDER_LABELS[providerKind] ?? providerKind
}

export function providerDisplayName(account: ProviderAccount): string {
  return (
    account.display_name ||
    account.label ||
    account.email ||
    (typeof account.config?.email === 'string' ? account.config.email : null) ||
    account.external_account_id ||
    account.account_id
  )
}

export function selectedAccountEmail(account: ProviderAccount): string {
  return account.email || (typeof account.config?.email === 'string' ? account.config.email : '')
}

export function providerAccountEnabled(account: ProviderAccount): boolean {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return false
  if (typeof account.is_active === 'boolean') return account.is_active
  return account.config?.auth_state !== 'logged_out'
}

export function defaultProviderIdFromAccount(
  providerKind: string | null | undefined
): ConnectionProviderId {
  switch (providerKind) {
    case 'telegram_user':
    case 'telegram_bot':
      return 'telegram'
    case 'whatsapp_web':
    case 'whatsapp_business_cloud':
      return 'whatsapp'
    case 'zoom_user':
    case 'zoom_server_to_server':
      return 'zoom'
    case 'yandex_telemost_user':
      return 'yandex_telemost'
    case 'zulip_bot':
      return 'zulip'
    default:
      return 'mail'
  }
}

export function downloadJson(filename: string, value: unknown): void {
  const blob = new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

export function matchesCalendarAccount(
  account: ProviderAccount,
  calendarAccount: CalendarAccount
): boolean {
  const accountEmail = selectedAccountEmail(account).toLowerCase()
  const calendarEmail = calendarAccount.email?.toLowerCase() ?? ''
  return (
    calendarAccount.account_id === account.account_id ||
    calendarAccount.account_id === account.external_account_id ||
    calendarAccount.account_name === account.display_name ||
    (accountEmail.length > 0 && calendarEmail === accountEmail)
  )
}
