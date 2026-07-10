import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import { useCommunicationMessagesQuery, useMailboxHealthQuery } from './useHomeQuery'
import type { FeedItem, PersonaItem, StatCard } from '../types/home'

const channelIcons: Record<string, string> = {
  email: 'tabler:mail',
  gmail: 'tabler:brand-gmail',
  icloud: 'tabler:cloud',
  imap: 'tabler:server',
  telegram_user: 'tabler:brand-telegram',
  telegram_bot: 'tabler:brand-telegram',
  whatsapp_web: 'tabler:brand-whatsapp'
}

export function useHomePageSurface() {
  const { t } = useI18n()
  const messagesQuery = useCommunicationMessagesQuery(50)
  const mailboxHealthQuery = useMailboxHealthQuery()

  const homeStats = computed<StatCard[]>(() => {
    const stats: StatCard[] = []
    const mailboxHealth = mailboxHealthQuery.data.value
    if (mailboxHealth) {
      stats.push({ label: t('Messages'), value: String(mailboxHealth.total_messages), delta: `+${mailboxHealth.unread}`, icon: 'tabler:mail' })
      stats.push({ label: t('Needs attention'), value: String(mailboxHealth.needs_action), delta: `+${mailboxHealth.important}`, icon: 'tabler:alert-circle' })
      stats.push({ label: t('Waiting'), value: String(mailboxHealth.waiting), delta: `${mailboxHealth.done} ${t('done')}`, icon: 'tabler:message-reply' })
    }
    stats.push({ label: t('Projects'), value: '—', delta: t('active'), icon: 'tabler:briefcase' })
    stats.push({ label: t('Personas'), value: '—', delta: t('enriched'), icon: 'tabler:user-plus' })
    return stats
  })

  const whatsNew = computed<FeedItem[]>(() => {
    const items: FeedItem[] = []
    const messages = messagesQuery.data.value ?? []
    for (const message of messages.slice(0, 5)) {
      const sender = message.sender_display_name || message.sender || t('Unknown')
      items.push({
        icon: channelIcons[message.channel_kind] || 'tabler:message',
        title: t('New message from {sender}').replace('{sender}', sender),
        meta: message.subject || message.body_text_preview,
        time: message.occurred_at || message.projected_at,
        tone: 'blue'
      })
    }
    return items
  })

  const personasTalked = computed<PersonaItem[]>(() => {
    const seen = new Set<string>()
    const result: PersonaItem[] = []
    const messages = messagesQuery.data.value ?? []
    for (const message of messages) {
      const sender = message.sender_display_name || message.sender || t('Unknown')
      if (seen.has(sender)) continue
      seen.add(sender)
      result.push({
        name: sender,
        meta: message.subject || message.body_text_preview,
        icon: 'tabler:message'
      })
      if (result.length >= 5) break
    }
    return result
  })

  return {
    homeStats,
    mailboxHealth: mailboxHealthQuery.data,
    messages: messagesQuery.data,
    personasTalked,
    whatsNew
  }
}
