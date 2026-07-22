import { computed } from 'vue'
import type { AISettingsSurface } from './useAISettingsSurface'
import { useI18n } from '../../../platform/i18n'
import {
  formatCompactNumber,
  formatCurrency,
  formatLatency,
  hourlyChartBuckets,
  maxHourlyRequestCount,
} from '../components/aiUsageStatsPresentation'
import { aiProviderBrand, providerBrandClass } from '../components/providerBranding'

export function useAIUsageStatsPanelController(options: {
  surface: AISettingsSurface
}) {
  const { t } = useI18n()

  const usageBuckets = computed(() => hourlyChartBuckets(options.surface.hourlyUsageRows.value))
  const maxHourlyRequests = computed(() => maxHourlyRequestCount(usageBuckets.value))

  function unknownLabel(): string {
    return t('Unknown')
  }

  function compact(value: number | null | undefined): string {
    return formatCompactNumber(value, unknownLabel())
  }

  function currency(value: number | null | undefined): string {
    return formatCurrency(value, unknownLabel())
  }

  function latency(value: number | null | undefined): string {
    return formatLatency(value, unknownLabel())
  }

  function providerIcon(providerKind: string, providerKey?: string): string {
    return aiProviderBrand(providerKind, providerKey).icon
  }

  function providerIconTone(providerKind: string, providerKey?: string): string {
    return providerBrandClass(aiProviderBrand(providerKind, providerKey))
  }

  function handleRefreshUsageStats(): void {
    void options.surface.handleRefreshUsageStats()
  }

  return {
    t,
    usageBuckets,
    maxHourlyRequests,
    compact,
    currency,
    latency,
    providerIcon,
    providerIconTone,
    handleRefreshUsageStats,
  }
}
