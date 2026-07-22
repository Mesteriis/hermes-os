import type { CalendarAccount, ProviderAccount } from '../types/settings'

export type TranslateFn = (key: string) => string

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
  return providerKind === 'whatsapp_web'
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

export function canOpenSetupAction(providerKind: string): boolean {
  return (
    providerKind === 'gmail' ||
    isTelegramProvider(providerKind) ||
    isWhatsappProvider(providerKind) ||
    isZoomProvider(providerKind) ||
    isYandexTelemostProvider(providerKind) ||
    isExceptionRouteProvider(providerKind)
  )
}

export function accountConfigBoolean(account: ProviderAccount, key: string): boolean | null {
  const value = account.config?.[key]
  return typeof value === 'boolean' ? value : null
}

export function accountContactsSyncEnabled(account: ProviderAccount): boolean {
  return accountConfigBoolean(account, 'address_book_sync_enabled') ?? true
}

export function accountCredentialRequiresReauthorization(account: ProviderAccount): boolean {
  return (
    account.credential_state?.status === 'expired' &&
    account.credential_state.requires_reauthorization === true
  )
}

export function accountConfigString(account: ProviderAccount, key: string): string | null {
  const value = account.config?.[key]
  return typeof value === 'string' ? value : null
}

export function accountSupportsContacts(account: ProviderAccount): boolean {
  const raw = account.config?.connected_services
  if (!Array.isArray(raw)) return false
  return raw.includes('contacts')
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

export function accountConnectedServices(account: ProviderAccount): string[] {
  const raw = account.config?.connected_services
  if (!Array.isArray(raw)) return []
  return raw.filter((service): service is string => typeof service === 'string')
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

export function selectedAccountEmail(account: ProviderAccount): string {
  return account.email || (typeof account.config?.email === 'string' ? account.config.email : '')
}

export function providerAccountEnabled(account: ProviderAccount): boolean {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) return false
  if (typeof account.is_active === 'boolean') return account.is_active
  return account.config?.auth_state !== 'logged_out'
}
