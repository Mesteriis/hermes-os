<script setup lang="ts">
import type { CommunicationMessageDetailResponse } from '../types/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const message = props.detail?.message ?? null
</script>

<template>
  <div class="headers-tab">
    <table v-if="message" class="headers-table">
      <tbody>
        <tr>
          <td class="header-label">From</td>
          <td class="header-value">{{ message.sender }}</td>
        </tr>
        <tr>
          <td class="header-label">To</td>
          <td class="header-value">{{ message.recipients?.join(', ') || '-' }}</td>
        </tr>
        <tr>
          <td class="header-label">Subject</td>
          <td class="header-value">{{ message.subject }}</td>
        </tr>
        <tr>
          <td class="header-label">Date</td>
          <td class="header-value">{{ message.projected_at || message.occurred_at || '-' }}</td>
        </tr>
        <tr>
          <td class="header-label">Channel</td>
          <td class="header-value">{{ message.channel_kind }}</td>
        </tr>
        <tr>
          <td class="header-label">Message ID</td>
          <td class="header-value monospace">{{ message.message_id }}</td>
        </tr>
        <tr>
          <td class="header-label">Account</td>
          <td class="header-value">{{ message.account_id }}</td>
        </tr>
        <tr>
          <td class="header-label">State</td>
          <td class="header-value">{{ message.workflow_state }} / {{ message.local_state }}</td>
        </tr>
        <tr>
          <td class="header-label">Importance</td>
          <td class="header-value">{{ message.importance_score ?? 'N/A' }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="no-data">No message selected</div>
  </div>
</template>

<style scoped>
.headers-tab {
  padding: 1rem;
}

.headers-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
}

.headers-table td {
  padding: 0.375rem 0.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  vertical-align: top;
}

.header-label {
  width: 120px;
  font-weight: 600;
  color: var(--hh-text-secondary, #6b7280);
  white-space: nowrap;
}

.header-value {
  color: var(--hh-text-primary, #1f2937);
  word-break: break-all;
}

.monospace {
  font-family: monospace;
  font-size: 0.75rem;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
