import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramStatusMessages realtime recovery status', () => {
  it('renders shared realtime state without owning transport or fetching data', () => {
    const source = readFileSync(new URL('./TelegramStatusMessages.vue', import.meta.url), 'utf8')

    expect(source).toContain('realtimeStatusLabel: string')
    expect(source).toContain('realtimeStatusDetail: string')
    expect(source).toContain("realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'")
    expect(source).toContain("{{ t('Realtime') }}: {{ realtimeStatusLabel }}")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('new WebSocket')
    expect(source).not.toContain('EventSource')
  })
})
