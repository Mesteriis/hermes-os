import type { ConnectionProviderId } from '../../../shared/stores/integrationConnectionWizard'
import type { CalendarAccount, ProviderAccount } from '../types/settings'
import {
  communicationProviderBrand,
  providerBrandClass,
} from '../components/providerBranding'
import type { TranslateFn } from './integrationAccountPredicates'
import {
  accountConfigBoolean,
  accountConfigString,
  accountContactsRemoteWriteEnabled,
  accountContactsSyncDirection,
  accountContactsSyncEnabled,
  accountCredentialRequiresReauthorization,
  accountHasGoogleContactsWriteScope,
  accountSupportsContacts,
  canOpenSetupAction as canOpenSetupActionForProviderKind,
  isExceptionOnlyProvider,
  isExceptionRouteProvider,
  isManagedRuntimeProvider,
  isMailProvider,
  isTelegramProvider,
  isWhatsappProvider,
  isYandexTelemostProvider,
  isZulipProvider,
  isZoomProvider,
  matchesCalendarAccount,
  providerAccountEnabled,
  selectedAccountEmail,
  supportsCalendar,
} from './integrationAccountPredicates'

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

export interface MailSyncSettingsSnapshot {
  sync_enabled?: boolean
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

export interface AccountServiceRowsInput {
  account: ProviderAccount
  selectedMailSyncSettings: MailSyncSettingsSnapshot | null
  calendarAccount: CalendarAccount | null
  activeMailActionAccountId: string | null
  isMailSyncUpdatePending: boolean
  isCalendarUpdatePending: boolean
  isProviderUpdatePending: boolean
  isRunAddressBookSyncNowPending: boolean
  t: TranslateFn
}

const PROVIDER_LABELS: Record<string, string> = {
  gmail: 'Gmail',
  icloud: 'iCloud',
  imap: 'IMAP',
  telegram_user: 'Telegram User',
  telegram_bot: 'Telegram Bot',
  whatsapp_web: 'WhatsApp Web',
  zoom_user: 'Zoom (OAuth/Live)',
  zoom_server_to_server: 'Zoom (Server-to-Server)',
  yandex_telemost_user: 'Yandex Telemost',
  zulip_bot: 'Zulip Bot',
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

export function defaultProviderIdFromAccount(
  providerKind: string | null | undefined
): ConnectionProviderId {
  switch (providerKind) {
    case 'telegram_user':
    case 'telegram_bot':
      return 'telegram'
    case 'whatsapp_web':
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

export function providerFlowLabel(providerKind: string, t: TranslateFn): string {
  if (providerKind === 'gmail') return t('Browser callback')
  if (isTelegramProvider(providerKind)) return t('QR companion')
  if (isExceptionOnlyProvider(providerKind)) return t('Exception-only recovery')
  if (isWhatsappProvider(providerKind)) return t('QR companion')
  if (isExceptionRouteProvider(providerKind)) return t('Exception route')
  if (isZoomProvider(providerKind) || isYandexTelemostProvider(providerKind)) {
    return t('Browser callback')
  }
  return t('Managed flow')
}

export function nextStepLabel(providerKind: string, t: TranslateFn): string {
  if (providerKind === 'gmail') return t('Resume in browser callback flow')
  if (isTelegramProvider(providerKind)) return t('Start or resume Telegram QR login')
  if (isExceptionOnlyProvider(providerKind)) {
    return t('Handle only through explicit exception recovery')
  }
  if (isWhatsappProvider(providerKind)) return t('Resume hidden WhatsApp runtime')
  if (isExceptionRouteProvider(providerKind)) {
    return t('Continue in dedicated runtime')
  }
  if (isZoomProvider(providerKind) || isYandexTelemostProvider(providerKind)) {
    return t('Resume through the workspace callback route')
  }
  return t('Use managed setup flow')
}

export function statusText(account: ProviderAccount, t: TranslateFn): string {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) {
    return t('Not authenticated')
  }
  if (typeof account.is_active === 'boolean' && !account.is_active) return t('Inactive')
  if (isManagedRuntimeProvider(account.provider_kind)) return t('Configured')
  return t('Active')
}

export function statusClass(account: ProviderAccount): string {
  if (typeof account.is_authenticated === 'boolean' && !account.is_authenticated) {
    return 'unauthenticated'
  }
  if (typeof account.is_active === 'boolean' && !account.is_active) return 'inactive'
  if (isManagedRuntimeProvider(account.provider_kind)) return 'configured'
  return 'active'
}

export function selectedInspectorActionLabel(account: ProviderAccount | null, t: TranslateFn): string {
  if (!account) return t('Open connection wizard')
  if (isWhatsappProvider(account.provider_kind)) return t('Resume QR companion')
  if (isTelegramProvider(account.provider_kind)) return t('Resume Telegram QR login')
  if (account.provider_kind === 'gmail') return t('Resume browser callback')
  if (isExceptionRouteProvider(account.provider_kind)) return t('View exception route')
  if (isExceptionOnlyProvider(account.provider_kind)) return t('No self-serve setup route')
  return t('Review managed route')
}

export function canOpenSetupAction(account: ProviderAccount): boolean {
  return canOpenSetupActionForProviderKind(account.provider_kind)
}

export function toIntegrationAccountRow(
  account: ProviderAccount,
  selectedIntegrationId: string | null,
  t: TranslateFn
): IntegrationAccountRow {
  const brand = communicationProviderBrand(account.provider_kind)
  return {
    account,
    displayName: providerDisplayName(account),
    icon: brand.icon,
    iconTone: providerBrandClass(brand),
    providerLabel: providerLabel(account.provider_kind),
    flowLabel: providerFlowLabel(account.provider_kind, t),
    statusText: statusText(account, t),
    statusClass: statusClass(account),
    isSelected: selectedIntegrationId === account.account_id,
  }
}

export function toIntegrationGroups(
  accounts: ProviderAccount[],
  selectedIntegrationId: string | null,
  t: TranslateFn
): IntegrationGroup[] {
  const groups = [
    {
      label: t('Mail accounts'),
      items: accounts.filter((account) => isMailProvider(account.provider_kind)),
    },
    {
      label: t('Telegram accounts'),
      items: accounts.filter((account) => isTelegramProvider(account.provider_kind)),
    },
    {
      label: t('WhatsApp accounts'),
      items: accounts.filter((account) => isWhatsappProvider(account.provider_kind)),
    },
    {
      label: t('Zoom accounts'),
      items: accounts.filter((account) => isZoomProvider(account.provider_kind)),
    },
    {
      label: t('Yandex Telemost accounts'),
      items: accounts.filter((account) => isYandexTelemostProvider(account.provider_kind)),
    },
    {
      label: t('Zulip accounts'),
      items: accounts.filter((account) => isZulipProvider(account.provider_kind)),
    },
    {
      label: t('Other accounts'),
      items: accounts.filter((account) =>
        !isMailProvider(account.provider_kind) &&
        !isTelegramProvider(account.provider_kind) &&
        !isWhatsappProvider(account.provider_kind) &&
        !isZoomProvider(account.provider_kind) &&
        !isYandexTelemostProvider(account.provider_kind) &&
        !isZulipProvider(account.provider_kind)
      ),
    },
  ]

  const nonEmptyGroups = groups
    .filter((group) => group.items.length > 0)
    .map((group) => ({
      label: group.label,
      items: group.items.map((account) => toIntegrationAccountRow(account, selectedIntegrationId, t)),
    }))

  if (nonEmptyGroups.length > 0) return nonEmptyGroups
  return [
    { label: t('Accounts'), items: accounts.map((account) => toIntegrationAccountRow(account, selectedIntegrationId, t)) },
  ]
}

export function toIntegrationServiceRows(input: AccountServiceRowsInput): AccountServiceRow[] {
  const rows: AccountServiceRow[] = []
  const { account, selectedMailSyncSettings, calendarAccount, activeMailActionAccountId } = input
  const { isMailSyncUpdatePending, isCalendarUpdatePending, isProviderUpdatePending, isRunAddressBookSyncNowPending, t } = input

  if (isMailProvider(account.provider_kind)) {
    rows.push({
      id: 'mail',
      label: t('Mail'),
      icon: 'tabler:mail',
      enabled: selectedMailSyncSettings?.sync_enabled ?? providerAccountEnabled(account),
      statusText: selectedMailSyncSettings?.sync_enabled ? t('Sync enabled') : t('Sync paused'),
      detail: selectedMailSyncSettings
        ? t('Uses backend mail sync settings for this account.')
        : t('Waiting for mail sync settings from the backend.'),
      canToggle: Boolean(selectedMailSyncSettings),
      canRunNow: false,
      disabledReason: selectedMailSyncSettings ? undefined : t('Sync settings are still loading.'),
      isBusy: isMailSyncUpdatePending && activeMailActionAccountId === account.account_id,
    })
  }

  if (supportsCalendar(account.provider_kind) || calendarAccount) {
    const enabled = Boolean(calendarAccount && calendarAccount.sync_status !== 'paused' && calendarAccount.sync_status !== 'disabled')
    rows.push({
      id: 'calendar',
      label: t('Calendar'),
      icon: 'tabler:calendar',
      enabled,
      statusText: calendarAccount ? calendarAccount.sync_status : t('Not configured'),
      detail: calendarAccount
        ? t('Uses the Calendar account sync_status contract.')
        : t('No matching Calendar account exists for this provider account.'),
      canToggle: Boolean(calendarAccount),
      canRunNow: false,
      disabledReason: calendarAccount ? undefined : t('Calendar account endpoint has no linked account yet.'),
      isBusy: isCalendarUpdatePending,
    })
  }

  const contactsSupported = accountSupportsContacts(account)
  const contactsEnabled = contactsSupported && accountContactsSyncEnabled(account)
  const contactsUnsupportedReason = accountConfigString(account, 'address_book_sync_unsupported_reason')
  const contactsDirection = accountContactsSyncDirection(account)
  const contactsRemoteWriteEnabled = accountContactsRemoteWriteEnabled(account)
  const contactsCanWrite = account.provider_kind === 'gmail' && accountHasGoogleContactsWriteScope(account)
  const contactsDetail = contactsSupported
    ? contactsUnsupportedReason
      ? t('Contacts sync is disabled for this account because the provider adapter is not available.')
      : contactsDirection === 'bidirectional'
        ? contactsRemoteWriteEnabled
          ? t('Two-way sync with provider contacts is enabled.')
          : contactsCanWrite
            ? t('Two-way sync is selected, but provider write is paused.')
            : t('Two-way sync is selected, but this account needs Contacts write permission before Hermes can push changes.')
        : t('Contacts sync reads provider contacts into Personas. Local changes are not pushed.')
    : t('Contacts are not provided by this integration.')
  rows.push({
    id: 'contacts',
    label: t('Contacts'),
    icon: 'tabler:address-book',
    enabled: contactsEnabled,
    statusText: contactsSupported
      ? contactsEnabled
        ? contactsDirection === 'bidirectional'
          ? t('Two-way sync')
          : t('Read-only sync')
        : contactsUnsupportedReason
          ? t('Not supported')
          : t('Sync paused')
      : t('Not provided'),
    detail: contactsDetail,
    canToggle: contactsSupported && !contactsUnsupportedReason,
    canRunNow: contactsSupported && !contactsUnsupportedReason,
    runNowLabel: t('Sync now'),
    modeActionLabel: contactsDirection === 'bidirectional' ? undefined : t('Enable two-way'),
    canRunModeAction: contactsSupported && !contactsUnsupportedReason && contactsDirection !== 'bidirectional',
    disabledReason: contactsSupported
      ? contactsUnsupportedReason
        ? t('Contacts sync is disabled for this account because the provider adapter is not available.')
        : undefined
      : t('Contacts are not provided by this integration.'),
    isBusy: (isProviderUpdatePending || isRunAddressBookSyncNowPending) && activeMailActionAccountId === account.account_id,
  })

  if (isTelegramProvider(account.provider_kind) || isWhatsappProvider(account.provider_kind)) {
    rows.push({
      id: 'messenger',
      label: t('Messenger'),
      icon: isWhatsappProvider(account.provider_kind) ? 'tabler:brand-whatsapp' : 'tabler:brand-telegram',
      enabled: providerAccountEnabled(account),
      statusText: statusText(account, t),
      detail: t('Runtime-owned messaging service; setup continues through the managed route.'),
      canToggle: false,
      canRunNow: false,
      disabledReason: t('Messenger runtime toggles are not exposed through Settings yet.'),
      isBusy: false,
    })
  }

  if (isZoomProvider(account.provider_kind) || isYandexTelemostProvider(account.provider_kind)) {
    rows.push({
      id: 'meetings',
      label: t('Meetings'),
      icon: 'tabler:video',
      enabled: providerAccountEnabled(account),
      statusText: statusText(account, t),
      detail: t('Meeting integration is runtime-owned and managed through its provider flow.'),
      canToggle: false,
      canRunNow: false,
      disabledReason: t('Meeting runtime toggle is not exposed through Settings yet.'),
      isBusy: false,
    })
  }

  return rows
}

export {
  accountConfigBoolean,
  accountContactsRemoteWriteEnabled,
  accountContactsSyncDirection,
  accountContactsSyncEnabled,
  accountConfigString,
  accountCredentialRequiresReauthorization,
  accountSupportsContacts,
  accountHasGoogleContactsWriteScope,
  isMailProvider,
  isTelegramProvider,
  isWhatsappProvider,
  isZoomProvider,
  isYandexTelemostProvider,
  isZulipProvider,
  matchesCalendarAccount,
  providerAccountEnabled,
  selectedAccountEmail,
}
