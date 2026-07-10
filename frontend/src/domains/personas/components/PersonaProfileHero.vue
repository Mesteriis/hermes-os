<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import { Avatar, Button, Icon } from '../../../shared/ui'
import type { EnrichedPersona, PersonaPanelProfile } from '../types/persona'
import {
  formatDateTime,
  healthScoreLabel,
  languageLabel,
  personaInitials,
  trustScoreLabel
} from './personaWorkspaceElements'

const props = withDefaults(defineProps<{
  selectedPersona: PersonaPanelProfile
  relationshipCount: number
  isRefreshing?: boolean
  settingOwnerPersonaId?: string | null
}>(), {
  isRefreshing: false,
  settingOwnerPersonaId: null
})

const emit = defineEmits<{
  refresh: []
  setOwner: [persona: EnrichedPersona]
}>()

const { t } = useI18n()

function isSettingOwner(personaId: string): boolean {
  return props.settingOwnerPersonaId === personaId
}
</script>

<template>
  <section class="personas-profile-hero">
    <div class="personas-hero-topline">
      <div class="personas-profile-title">
        <span class="personas-avatar-ring">
          <Avatar
            size="xl"
            :fallback="personaInitials(selectedPersona)"
            :alt="selectedPersona.display_name"
          />
          <i aria-hidden="true" />
        </span>
        <div>
          <h2>{{ selectedPersona.display_name }}</h2>
          <p>{{ selectedPersona.tone || t('Relationship profile') }}</p>
          <ul class="personas-hero-meta" :aria-label="t('Persona metadata')">
            <li>
              <Icon icon="tabler:language" />
              {{ languageLabel(selectedPersona.language, t) }}
            </li>
            <li>
              <Icon icon="tabler:message-circle" />
              {{ selectedPersona.preferred_channel || t('Not set') }}
            </li>
            <li>
              <Icon icon="tabler:clock" />
              {{ formatDateTime(selectedPersona.last_interaction_at, t) }}
            </li>
          </ul>
          <ul class="personas-profile-tags" :aria-label="t('Topics')">
            <li v-for="topic in selectedPersona.frequent_topics" :key="topic">{{ topic }}</li>
            <li v-if="selectedPersona.is_owner">{{ t('Owner') }}</li>
          </ul>
        </div>
      </div>

      <div class="personas-hero-actions">
        <Button type="button" size="sm" icon="tabler:mail">
          {{ t('Write') }}
        </Button>
        <Button type="button" size="sm" variant="secondary" icon="tabler:calendar-plus">
          {{ t('Schedule meeting') }}
        </Button>
        <Button
          v-if="!selectedPersona.is_owner"
          type="button"
          size="sm"
          variant="outline"
          icon="tabler:user-check"
          :loading="isSettingOwner(selectedPersona.persona_id)"
          @click="emit('setOwner', selectedPersona)"
        >
          {{ t('Mark as me') }}
        </Button>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          icon="tabler:refresh"
          :aria-label="t('Refresh')"
          :loading="isRefreshing"
          @click="emit('refresh')"
        />
      </div>
    </div>

    <dl class="personas-score-strip">
      <div>
        <dt>{{ t('Trust') }}</dt>
        <dd>
          <span>{{ trustScoreLabel(selectedPersona.trust_score, t) }}</span>
          <small>{{ t('Reliable') }}</small>
        </dd>
      </div>
      <div>
        <dt>{{ t('Health') }}</dt>
        <dd>
          <span>{{ healthScoreLabel(selectedPersona.trust_score, t) }}</span>
          <small>{{ t('Stable') }}</small>
        </dd>
      </div>
      <div>
        <dt>{{ t('Engagement') }}</dt>
        <dd>
          <span>{{ selectedPersona.interaction_count }}</span>
          <small>{{ t('Interactions') }}</small>
        </dd>
      </div>
      <div>
        <dt>{{ t('Last interaction') }}</dt>
        <dd>
          <span>{{ formatDateTime(selectedPersona.last_interaction_at, t) }}</span>
          <small>{{ selectedPersona.preferred_channel || t('Not set') }}</small>
        </dd>
      </div>
      <div>
        <dt>{{ t('Relationship') }}</dt>
        <dd>
          <span>{{ relationshipCount }}</span>
          <small>{{ t('Known links') }}</small>
        </dd>
      </div>
    </dl>
  </section>
</template>
