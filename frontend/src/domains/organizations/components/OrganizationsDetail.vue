<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { Organization } from '../types/organization'

const { t } = useI18n()

defineProps<{
  selectedOrganization: Record<string, unknown> | null
  orgPeople: unknown[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="organizations-detail">
    <section class="panel org-detail-panel">
      <template v-if="selectedOrganization">
        <header>
          <span class="round-icon blue"><Icon icon="tabler:building" :size="26" /></span>
          <div>
            <h2>{{ selectedOrganization.display_name as string }}</h2>
            <em>{{ selectedOrganization.industry as string || t('Unknown industry') }}{{ selectedOrganization.country ? ` · ${selectedOrganization.country as string}` : '' }}</em>
          </div>
        </header>
        <div class="org-detail-grid">
          <div class="info-card">
            <h3>{{ t('Status') }}</h3>
            <span class="status-chip">{{ selectedOrganization.status as string }}</span>
            <span v-if="selectedOrganization.health_status" class="health-chip">{{ selectedOrganization.health_status as string }}</span>
            <span v-if="selectedOrganization.watchlist" class="health-chip important">{{ t('Watchlist') }}</span>
          </div>
          <div v-if="selectedOrganization.description" class="info-card">
            <h3>{{ t('About') }}</h3>
            <p>{{ selectedOrganization.description as string }}</p>
          </div>
          <div class="info-card">
            <h3>{{ t('Details') }}</h3>
            <div v-if="selectedOrganization.website" class="detail-row">
              <span>{{ t('Website') }}</span>
              <strong>{{ selectedOrganization.website as string }}</strong>
            </div>
            <div v-if="selectedOrganization.legal_name" class="detail-row">
              <span>{{ t('Legal name') }}</span>
              <strong>{{ selectedOrganization.legal_name as string }}</strong>
            </div>
            <div v-if="selectedOrganization.registration_number" class="detail-row">
              <span>{{ t('Registration') }}</span>
              <strong>{{ selectedOrganization.registration_number as string }}</strong>
            </div>
            <div v-if="selectedOrganization.vat" class="detail-row">
              <span>{{ t('VAT') }}</span>
              <strong>{{ selectedOrganization.vat as string }}</strong>
            </div>
            <div class="detail-row">
              <span>{{ t('Interactions') }}</span>
              <strong>{{ selectedOrganization.interaction_count as string }}</strong>
            </div>
            <div class="detail-row">
              <span>{{ t('Priority') }}</span>
              <strong>{{ selectedOrganization.priority as string || t('normal') }}</strong>
            </div>
          </div>
          <div v-if="orgPeople.length > 0" class="info-card">
            <h3>{{ t('Key People') }}</h3>
            <div v-for="person in orgPeople" :key="(person as Record<string, unknown>).person_id as string" class="person-mini">
              <span class="round-icon"><Icon icon="tabler:user" :size="16" /></span>
              <strong>{{ (person as Record<string, unknown>).display_name as string }}</strong>
              <small>{{ (person as Record<string, unknown>).email_address as string }}</small>
            </div>
          </div>
        </div>
      </template>
      <template v-else>
        <header>
          <span class="round-icon"><Icon icon="tabler:building-off" :size="26" /></span>
          <div>
            <h2>{{ t('No company selected') }}</h2>
            <em>{{ t('Select a company from the list') }}</em>
          </div>
        </header>
      </template>
    </section>
  </div>
</template>

<style scoped>
.org-detail-panel {
  padding: 12px;
}
.org-detail-panel header {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}
.org-detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}
.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
  font-size: 12px;
}
.detail-row span {
  color: var(--hh-color-text-muted);
}
.person-mini {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
  align-items: center;
  padding: 6px 0;
  border-top: 1px solid rgba(102, 189, 180, 0.08);
}
</style>
