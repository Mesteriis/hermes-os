import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramStatusMessages realtime recovery status', () => {
  it('renders shared realtime state without owning transport or fetching data', () => {
    const source = readFileSync(new URL('./TelegramStatusMessages.vue', import.meta.url), 'utf8')

    expect(source).toContain('realtimeStatusLabel: string')
    expect(source).toContain('realtimeStatusDetail: string')
    expect(source).toContain('realtimeRecoveryDetail: string')
    expect(source).toContain("realtimeStatusTone: 'neutral' | 'success' | 'warning' | 'danger'")
    expect(source).toContain('useRealtimeStatusStore')
    expect(source).toContain('realtimeStatus.canTriggerReconnect')
    expect(source).toContain("t('Reconnect realtime')")
    expect(source).toContain('realtimeStatus.requestReconnect()')
    expect(source).toContain("{{ t('Realtime') }}: {{ realtimeStatusLabel }}")
    expect(source).toContain("{{ t('Recovery') }}: {{ realtimeRecoveryDetail }}")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('new WebSocket')
    expect(source).not.toContain('EventSource')
  })
})
