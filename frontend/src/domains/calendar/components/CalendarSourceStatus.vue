<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { CalendarAccount, CalendarSource } from '../types/calendar'

const { t } = useI18n()

defineProps<{
  calendarSources: CalendarSource[]
  calendarAccounts: CalendarAccount[]
}>()
</script>

<template>
  <section class="panel info-card">
    <h2>{{ t('Calendars') }}</h2>
    <template v-if="calendarSources.length === 0">
      <label v-for="acct in calendarAccounts" :key="acct.account_id" class="mini-check">
        <input type="checkbox" checked disabled />{{ acct.account_name }}<em>{{ acct.provider }}</em>
      </label>
    </template>
    <template v-else>
      <label v-for="src in calendarSources" :key="src.source_id" class="mini-check">
        <input type="checkbox" checked disabled />{{ src.name }}<em>{{ src.timezone || '' }}</em>
      </label>
    </template>
  </section>
</template>
