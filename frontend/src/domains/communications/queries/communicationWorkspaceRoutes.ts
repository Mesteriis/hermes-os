export type PrimaryChannelId = 'mail' | 'telegram' | 'whatsapp'

const primaryChannelIds: readonly PrimaryChannelId[] = [
  'mail',
  'telegram',
  'whatsapp',
]

export function routeToChannelId(
  routeId: string | undefined
): PrimaryChannelId | undefined {
  if (!routeId || routeId === 'communications') return 'mail'

  for (const channelId of primaryChannelIds) {
    if (routeId === `communications-${channelId}`) return channelId
    if (routeId === `communications-${channelId}-accounts:all`)
      return channelId
    if (routeId.startsWith(`communications-${channelId}-account:`))
      return channelId
  }

  return undefined
}

export function routeToAccountId(
  routeId: string | undefined
): string | undefined {
  if (!routeId) return undefined
  const accountMarker = '-account:'
  const markerIndex = routeId.indexOf(accountMarker)
  if (markerIndex < 0) return undefined

  try {
    return decodeURIComponent(routeId.slice(markerIndex + accountMarker.length))
  } catch {
    return undefined
  }
}
