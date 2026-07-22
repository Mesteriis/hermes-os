import type { AiHubHourlyUsageStats } from '../types/aiControlCenter'

export interface AiHubHourlyChartBucket {
  hour: string
  label: string
  requestCount: number
  failedCount: number
  estimatedTokens: number
}

export function hourlyChartBuckets(
  rows: AiHubHourlyUsageStats[],
  providerId?: string
): AiHubHourlyChartBucket[] {
  const now = new Date()
  const hours: AiHubHourlyChartBucket[] = []
  for (let index = 23; index >= 0; index -= 1) {
    const hour = new Date(now)
    hour.setMinutes(0, 0, 0)
    hour.setHours(now.getHours() - index)
    const key = hour.toISOString().slice(0, 13)
    const matchingRows = rows.filter((row) =>
      row.hour.slice(0, 13) === key && (!providerId || row.provider_id === providerId)
    )
    hours.push({
      hour: key,
      label: hour.toLocaleTimeString([], { hour: '2-digit' }),
      requestCount: sumRows(matchingRows, 'request_count'),
      failedCount: sumRows(matchingRows, 'failed_count'),
      estimatedTokens: sumRows(matchingRows, 'estimated_tokens'),
    })
  }
  return hours
}

export function maxHourlyRequestCount(
  buckets: AiHubHourlyChartBucket[]
): number {
  let maxRequests = 1
  for (const bucket of buckets) {
    if (bucket.requestCount > maxRequests) maxRequests = bucket.requestCount
  }
  return maxRequests
}

export function formatCompactNumber(
  value: number | null | undefined,
  unknownLabel: string
): string {
  if (typeof value !== 'number') return unknownLabel
  return new Intl.NumberFormat(undefined, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(value)
}

export function formatCurrency(value: number | null | undefined, unknownLabel: string): string {
  if (typeof value !== 'number') return unknownLabel
  return new Intl.NumberFormat(undefined, {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 4,
  }).format(value)
}

export function formatLatency(value: number | null | undefined, unknownLabel: string): string {
  if (typeof value !== 'number') return unknownLabel
  return `${Math.round(value)} ms`
}

function sumRows(
  rows: AiHubHourlyUsageStats[],
  key: 'request_count' | 'failed_count' | 'estimated_tokens'
): number {
  return rows.reduce((total, row) => total + row[key], 0)
}
