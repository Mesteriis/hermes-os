import type { CommunicationSubSurfaceId } from '../../domains/communications/queries/communicationChannelSurface'
import { isMailProviderKind } from '../../domains/communications/helpers/mailProviderKinds'
import type { ProviderAccount } from '../../domains/settings/types/settings'
import type { TelegramAccount } from '../../integrations/telegram/types/telegram'
import type { WhatsappAccountSummary } from '../../integrations/whatsapp/types/whatsapp'

export type AppLayoutNavbarNavigationIconTone =
  | 'accounts'
  | 'calendar'
  | 'channels'
  | 'communication'
  | 'dashboard'
  | 'documents'
  | 'knowledge'
  | 'mail'
  | 'review'
  | 'settings'
  | 'tasks'
  | 'telegram'
  | 'whatsapp'

export type AppLayoutNavbarRouteNode = {
  id: string
  label: string
  icon?: string
  iconTone?: AppLayoutNavbarNavigationIconTone
  children?: readonly AppLayoutNavbarRouteNode[]
}

export type AppLayoutNavbarAccountNavigationState = {
  mail: readonly ProviderAccount[]
  telegram: readonly TelegramAccount[]
  whatsapp: readonly WhatsappAccountSummary[]
}

const communicationsNavbarSurfaceIds = [
  'mail',
  'telegram',
  'whatsapp',
] as const satisfies readonly CommunicationSubSurfaceId[]

export type AppLayoutNavbarCommunicationSurfaceId =
  (typeof communicationsNavbarSurfaceIds)[number]

export function isCommunicationsNavbarSurfaceId(
  channelId: CommunicationSubSurfaceId
): channelId is AppLayoutNavbarCommunicationSurfaceId {
  return (
    communicationsNavbarSurfaceIds as readonly CommunicationSubSurfaceId[]
  ).includes(channelId)
}

export function isCommunicationChannelRouteId(routeId: string): boolean {
  return communicationsNavbarSurfaceIds.some(
    (channelId) => routeId === `communications-${channelId}`
  )
}

export function communicationAccountRouteNodes(
  channelId: AppLayoutNavbarCommunicationSurfaceId,
  accountNavigation: AppLayoutNavbarAccountNavigationState
): AppLayoutNavbarRouteNode[] {
  if (channelId === 'mail') {
    const accounts: AppLayoutNavbarRouteNode[] = accountNavigation.mail.map((account) => ({
      id: accountRouteId(channelId, account.account_id),
      label: mailAccountLabel(account),
      icon: 'tabler:mail-opened',
      iconTone: 'mail',
    }))
    return accounts.length > 0
      ? [allAccountsRouteNode(channelId, 'Все ящики'), ...accounts]
      : []
  }

  if (channelId === 'telegram') {
    const accounts: AppLayoutNavbarRouteNode[] = accountNavigation.telegram.map((account) => ({
      id: accountRouteId(channelId, account.account_id),
      label: providerRuntimeAccountLabel(account),
      icon: 'tabler:user-circle',
      iconTone: 'telegram',
    }))
    return accounts.length > 0
      ? [allAccountsRouteNode(channelId, 'Все аккаунты'), ...accounts]
      : []
  }

  const accounts: AppLayoutNavbarRouteNode[] = accountNavigation.whatsapp.map((account) => ({
    id: accountRouteId(channelId, account.account_id),
    label: providerRuntimeAccountLabel(account),
    icon: 'tabler:user-circle',
    iconTone: 'whatsapp',
  }))
  return accounts.length > 0
    ? [allAccountsRouteNode(channelId, 'Все аккаунты'), ...accounts]
    : []
}

export function isVisibleMailAccount(account: ProviderAccount): boolean {
  return account.is_active !== false && isMailProviderKind(account.provider_kind)
}

export function isVisibleTelegramAccount(account: TelegramAccount): boolean {
  return account.lifecycle_state !== 'removed'
}

export function isVisibleWhatsappAccount(account: WhatsappAccountSummary): boolean {
  return account.lifecycle_state !== 'removed'
}

export function emptyAccountNavigation(): AppLayoutNavbarAccountNavigationState {
  return {
    mail: [],
    telegram: [],
    whatsapp: [],
  }
}

export function communicationRouteIconTone(
  channelId: CommunicationSubSurfaceId
): AppLayoutNavbarNavigationIconTone {
  if (channelId === 'mail') return 'mail'
  if (channelId === 'telegram') return 'telegram'
  if (channelId === 'whatsapp') return 'whatsapp'

  return 'channels'
}

function allAccountsRouteNode(
  channelId: AppLayoutNavbarCommunicationSurfaceId,
  label: string
): AppLayoutNavbarRouteNode {
  return {
    id: allAccountsRouteId(channelId),
    label,
    icon: channelId === 'mail' ? 'tabler:inbox' : 'tabler:users',
    iconTone: communicationRouteIconTone(channelId),
  }
}

function allAccountsRouteId(channelId: AppLayoutNavbarCommunicationSurfaceId): string {
  return `communications-${channelId}-accounts:all`
}

function accountRouteId(
  channelId: AppLayoutNavbarCommunicationSurfaceId,
  accountId: string
): string {
  return `communications-${channelId}-account:${encodeURIComponent(accountId)}`
}

function mailAccountLabel(account: ProviderAccount): string {
  return firstNonEmpty([
    account.label,
    account.email,
    account.display_name,
    account.external_account_id,
    account.account_id,
  ])
}

function providerRuntimeAccountLabel(
  account: TelegramAccount | WhatsappAccountSummary
): string {
  return firstNonEmpty([
    account.display_name,
    account.external_account_id,
    account.account_id,
  ])
}

function firstNonEmpty(values: readonly (string | null | undefined)[]): string {
  for (const value of values) {
    const label = value?.trim()
    if (label) return label
  }

  return 'Account'
}
