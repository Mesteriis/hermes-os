<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AISettingsSurface } from '../queries/useAISettingsSurface'
import { useAIUsageStatsPanelController } from '../queries/useAIUsageStatsPanelController'

const props = defineProps<{
  surface: AISettingsSurface
}>()

const {
  t,
  usageBuckets,
  maxHourlyRequests,
  compact,
  currency,
  latency,
  providerIcon,
  providerIconTone,
  handleRefreshUsageStats,
} = useAIUsageStatsPanelController({
  surface: props.surface,
})
</script>

<template>
  <section class="settings-ai-usage-board">
    <header class="settings-ai-route-board__header">
      <div>
        <span>{{ t('Provider usage') }}</span>
        <strong>{{ t('Last 24 hours') }}</strong>
      </div>
      <div class="settings-ai-route-board__header-actions">
        <small>{{ t('AI Hub records route, provider, model, latency and estimated tokens without storing prompts.') }}</small>
        <button
          type="button"
          class="secondary-button"
          :disabled="surface.usageStatsQuery.isFetching.value"
          @click="handleRefreshUsageStats"
        >
          <Icon icon="tabler:refresh" />
          {{ t('Refresh') }}
        </button>
      </div>
    </header>

    <div class="settings-ai-usage-summary">
      <article>
        <span>{{ t('Requests') }}</span>
        <strong>{{ compact(surface.usageStats.value?.totals.request_count) }}</strong>
      </article>
      <article>
        <span>{{ t('Failed') }}</span>
        <strong>{{ compact(surface.usageStats.value?.totals.failed_count) }}</strong>
      </article>
      <article>
        <span>{{ t('Estimated tokens') }}</span>
        <strong>{{ compact(surface.usageStats.value?.totals.estimated_tokens) }}</strong>
      </article>
      <article>
        <span>{{ t('Estimated cost') }}</span>
        <strong>{{ currency(surface.usageStats.value?.totals.estimated_cost_usd) }}</strong>
      </article>
      <article>
        <span>{{ t('Average latency') }}</span>
        <strong>{{ latency(surface.usageStats.value?.totals.avg_latency_ms) }}</strong>
      </article>
    </div>

    <section class="settings-ai-usage-chart" :aria-label="t('Hourly AI activity')">
      <header>
        <h4>{{ t('Activity by hour') }}</h4>
        <small>{{ t('Bars show request count; labels include estimated tokens.') }}</small>
      </header>
      <div class="settings-ai-usage-bars">
        <article
          v-for="bucket in usageBuckets"
          :key="bucket.hour"
          class="settings-ai-usage-bar"
        >
          <progress :max="maxHourlyRequests" :value="bucket.requestCount" />
          <span>{{ bucket.label }}</span>
          <small>{{ bucket.requestCount }} / {{ compact(bucket.estimatedTokens) }}</small>
        </article>
      </div>
    </section>

    <section class="settings-ai-provider-usage-list">
      <article
        v-for="provider in surface.providerUsageRows.value"
        :key="provider.provider_id"
        class="settings-ai-provider-usage-card"
      >
        <header>
          <span
            class="settings-provider-icon"
            :class="providerIconTone(provider.provider_kind, provider.provider_key)"
          >
            <Icon :icon="providerIcon(provider.provider_kind, provider.provider_key)" />
          </span>
          <div>
            <strong>{{ provider.display_name }}</strong>
            <small>{{ provider.provider_key }} · {{ provider.status }}</small>
          </div>
        </header>
        <dl>
          <div>
            <dt>{{ t('Requests') }}</dt>
            <dd>{{ compact(provider.request_count) }}</dd>
          </div>
          <div>
            <dt>{{ t('Estimated tokens') }}</dt>
            <dd>{{ compact(provider.estimated_tokens) }}</dd>
          </div>
          <div>
            <dt>{{ t('Cost') }}</dt>
            <dd>{{ currency(provider.estimated_cost_usd) }}</dd>
          </div>
          <div>
            <dt>{{ t('Balance') }}</dt>
            <dd>{{ currency(provider.balance_remaining_usd) }}</dd>
          </div>
          <div>
            <dt>{{ t('Token quota') }}</dt>
            <dd>{{ compact(provider.token_quota_remaining) }}</dd>
          </div>
          <div>
            <dt>{{ t('Latency') }}</dt>
            <dd>{{ latency(provider.avg_latency_ms) }}</dd>
          </div>
        </dl>
      </article>
    </section>
  </section>
</template>
