<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from '../../../platform/i18n'
import HomeMetrics from '../components/HomeMetrics.vue'
import HomeWhatsNew from '../components/HomeWhatsNew.vue'
import HomePriorities from '../components/HomePriorities.vue'
import HomeUpcoming from '../components/HomeUpcoming.vue'
import HomePeopleTalked from '../components/HomePeopleTalked.vue'
import HomeSystemStatus from '../components/HomeSystemStatus.vue'
import HomeActiveProjects from '../components/HomeActiveProjects.vue'
import { useCommunicationMessagesQuery, useMailboxHealthQuery } from '../queries/useHomeQuery'
import type { StatCard, FeedItem, PersonItem, ProjectItem, TaskItem } from '../types/home'

const { t } = useI18n()
const router = useRouter()

const { data: messages } = useCommunicationMessagesQuery(50)
const { data: mailboxHealth } = useMailboxHealthQuery()

const channelIcons: Record<string, string> = {
  email: 'tabler:mail',
  gmail: 'tabler:brand-gmail',
  icloud: 'tabler:cloud',
  imap: 'tabler:server',
  telegram_user: 'tabler:brand-telegram',
  telegram_bot: 'tabler:brand-telegram',
  whatsapp_web: 'tabler:brand-whatsapp'
}

const homeStats = computed<StatCard[]>(() => {
  const stats: StatCard[] = []
  if (mailboxHealth.value) {
    stats.push({ label: t('Messages'), value: String(mailboxHealth.value.total_messages), delta: `+${mailboxHealth.value.unread}`, icon: 'tabler:mail' })
    stats.push({ label: t('Needs attention'), value: String(mailboxHealth.value.needs_action), delta: `+${mailboxHealth.value.important}`, icon: 'tabler:alert-circle' })
    stats.push({ label: t('Waiting'), value: String(mailboxHealth.value.waiting), delta: `${mailboxHealth.value.done} ${t('done')}`, icon: 'tabler:message-reply' })
  }
  stats.push({ label: t('Projects'), value: '—', delta: t('active'), icon: 'tabler:briefcase' })
  stats.push({ label: t('Persons'), value: '—', delta: t('enriched'), icon: 'tabler:user-plus' })
  return stats
})

const whatsNew = computed<FeedItem[]>(() => {
  const items: FeedItem[] = []
  const msgs = messages.value ?? []
  for (const msg of msgs.slice(0, 5)) {
    const sender = msg.sender_display_name || msg.sender || t('Unknown')
    items.push({
      icon: channelIcons[msg.channel_kind] || 'tabler:message',
      title: t('New message from {sender}').replace('{sender}', sender),
      meta: msg.subject || msg.body_text_preview,
      time: msg.occurred_at || msg.projected_at,
      tone: 'blue'
    })
  }
  return items
})

const peopleTalked = computed<PersonItem[]>(() => {
  const seen = new Set<string>()
  const result: PersonItem[] = []
  const msgs = messages.value ?? []
  for (const msg of msgs) {
    const sender = msg.sender_display_name || msg.sender || t('Unknown')
    if (seen.has(sender)) continue
    seen.add(sender)
    result.push({
      name: sender,
      meta: msg.subject || msg.body_text_preview,
      icon: 'tabler:message'
    })
    if (result.length >= 5) break
  }
  return result
})

function navigateToProjects() {
  router.push({ name: 'projects' })
}
</script>

<template>
  <section class="home-page">
    <div class="hero-row">
      <HomeMetrics :stats="homeStats" />
    </div>

    <div class="dashboard-grid">
      <HomeWhatsNew :items="whatsNew" />
      <HomePriorities :tasks="[] as TaskItem[]" />
      <HomeUpcoming />

      <aside class="stacked-rail">
        <HomePeopleTalked :people="peopleTalked" />
        <HomeSystemStatus statusError="" />
      </aside>
    </div>

    <HomeActiveProjects :projects="[] as ProjectItem[]" @navigate-to-projects="navigateToProjects" />
  </section>
</template>

<style scoped>
.home-page {
  padding: 18px;
  display: grid;
  gap: 14px;
}

.hero-row {
  --hh-zone-rows: 3;
  display: grid;
  grid-template-columns: 300px minmax(640px, 1fr);
  align-items: center;
  gap: 14px;
  min-height: var(--hh-widget-card);
}

.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 280px;
  gap: 14px;
}

.stacked-rail {
  display: grid;
  gap: 14px;
  align-content: start;
}

@media (max-width: 1359px) {
  .hero-row {
    grid-template-columns: 1fr;
  }
  .dashboard-grid {
    grid-template-columns: 1fr;
  }
}
</style>
