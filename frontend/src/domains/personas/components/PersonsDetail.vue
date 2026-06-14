<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { dossierSectionPreview } from '../stores/personas'
import type { PersonItem, PersonDossier } from '../types/persona'

const { t } = useI18n()

defineProps<{
  selectedPerson: PersonItem | null
  personDossier: PersonDossier | null
  isPersonDossierLoading: boolean
  personDossierError: string
  whatsNew: any[]
  projects: any[]
}>()
</script>

<template>
  <section class="person-detail">
    <template v-if="selectedPerson">
      <div class="widget-frame" data-widget-id="persons-hero">
        <section class="panel person-hero">
          <span class="round-icon ghost large-avatar"><Icon icon="tabler:user" :size="40" /></span>
          <div>
            <h1>{{ selectedPerson.name }}</h1>
            <p>{{ selectedPerson.role }} at {{ selectedPerson.company }}</p>
            <small>{{ selectedPerson.status ?? selectedPerson.channel ?? 'Contact' }}</small>
          </div>
          <div class="chat-actions">
            <button type="button" disabled><Icon icon="tabler:mail" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:phone" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:video" :size="17" /></button>
            <button type="button" disabled><Icon icon="tabler:brand-whatsapp" :size="17" /></button>
          </div>
        </section>
      </div>
      <div class="section-tabs">
        <button type="button" class="active">Overview</button>
        <button type="button" disabled>Communications</button>
        <button type="button" disabled>Documents <em>24</em></button>
        <button type="button" disabled>Tasks <em>7</em></button>
        <button type="button" disabled>Projects <em>5</em></button>
        <button type="button" disabled>Notes</button>
      </div>
      <div class="person-cards">
        <div class="widget-frame" data-widget-id="persons-information">
          <section class="panel info-card">
            <h2>Person Information</h2>
            <ul class="detail-list">
              <li><Icon icon="tabler:mail" :size="17" /> {{ selectedPerson.company }} <em>Primary</em></li>
              <li><Icon icon="tabler:phone" :size="17" /> +1 (555) 123-4567 <em>Mobile</em></li>
              <li><Icon icon="tabler:brand-telegram" :size="17" /> @john.smith <em>Telegram</em></li>
              <li><Icon icon="tabler:map-pin" :size="17" /> New York, USA <em>Local Time: 18:42</em></li>
            </ul>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-about">
          <section class="panel info-card">
            <h2>Persona Dossier</h2>
            <p v-if="isPersonDossierLoading">Loading dossier...</p>
            <p v-else-if="personDossierError" class="inline-error">{{ personDossierError }}</p>
            <template v-else-if="personDossier">
              <p>{{ personDossier.summary || 'No dossier summary yet.' }}</p>
              <div v-if="dossierSectionPreview(personDossier).length" class="tag-cloud">
                <span v-for="item in dossierSectionPreview(personDossier)" :key="item">{{ item }}</span>
              </div>
              <small>{{ personDossier.source_refs.length }} source refs · generated {{ new Date(personDossier.generated_at).toLocaleString() }}</small>
            </template>
            <p v-else>No dossier generated yet.</p>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-relationship-strength">
          <section class="panel info-card">
            <h2>Relationship Strength</h2>
            <div class="big-score">85</div>
            <strong>Strong</strong>
            <p>Last interaction 2 hours ago</p>
          </section>
        </div>
        <div class="widget-frame span-2" data-widget-id="persons-recent-interactions">
          <section class="panel info-card span-2">
            <h2>Recent Interactions</h2>
            <div v-for="item in whatsNew.slice(0, 3)" :key="item.title" class="feed-row compact-row">
              <span class="round-icon" :class="item.tone"><Icon :icon="item.icon" :size="18" /></span>
              <div>
                <strong>{{ item.title }}</strong>
                <p>{{ item.meta }}</p>
              </div>
              <time>{{ item.time }}</time>
            </div>
          </section>
        </div>
        <div class="widget-frame" data-widget-id="persons-active-projects">
          <section class="panel info-card">
            <h2>{{ t('Active Projects') }}</h2>
            <div v-for="project in projects.slice(0, 3)" :key="project.name" class="related-row">
              <span class="round-icon" :class="project.tone"><Icon :icon="project.icon" :size="16" /></span>
              <strong>{{ project.name }}</strong>
              <em>{{ project.progress }}%</em>
            </div>
          </section>
        </div>
      </div>
    </template>
    <template v-else>
      <section class="panel empty-domain-state">
        <Icon icon="tabler:user" :size="42" />
        <div>
          <h2>No person selected</h2>
          <p>Hermes Hub will show relationship memory here after persons are imported from local sources.</p>
        </div>
      </section>
    </template>
  </section>
</template>

<style scoped>
.person-detail {
  display: grid;
  gap: 12px;
  align-content: start;
  min-width: 0;
}
.person-hero {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 12px;
  min-height: var(--hh-widget-card-compact);
  border-bottom: 1px solid rgba(102, 189, 180, 0.12);
  padding: 12px 16px;
}
.large-avatar {
  width: 92px;
  height: 92px;
  border-radius: var(--hh-radius-round);
  display: flex;
  align-items: center;
  justify-content: center;
}
.person-cards {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}
</style>
