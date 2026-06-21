import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramRuntimePanel realtime boundary', () => {
	it('relies on the global realtime bootstrap instead of opening a panel-level telegram socket', () => {
		const source = readFileSync(new URL('./TelegramRuntimePanel.vue', import.meta.url), 'utf8')

		expect(source).not.toContain('createTelegramRealtimeConnection')
		expect(source).not.toContain('realtimeCleanup')
		expect(source).not.toContain('onMounted(() =>')
		expect(source).not.toContain('onUnmounted(() =>')
		expect(source).toContain('useRealtimeStatusStore()')
		expect(source).toContain('useTelegramAccountsQuery()')
		expect(source).toContain('useTelegramCapabilitiesQuery()')
		expect(source).toContain('useTelegramRuntimeStatusQuery(')
		expect(source).toContain('useStopTelegramRuntimeMutation()')
		expect(source).toContain('useStartTelegramRuntimeMutation()')
		expect(source).toContain('useRestartTelegramRuntimeMutation()')
		expect(source).toContain(':title="realtimeStatus.realtimeStatusDetail"')
		expect(source).toContain(':class="realtimeStatus.realtimeStatusTone"')
		expect(source).toContain("setTelegramRuntime('start')")
		expect(source).toContain("setTelegramRuntime('stop')")
		expect(source).toContain("setTelegramRuntime('restart')")
		expect(source).not.toContain('useTelegramMessagesQuery(')
		expect(source).not.toContain('useTelegramMessageSearchQuery(')
		expect(source).not.toContain('useTelegramMediaSearchQuery(')
		expect(source).not.toContain('useTelegramSendActions(')
		expect(source).not.toContain('Telegram' + 'Message' + 'Thread')
		expect(source).not.toContain('Telegram' + 'Chat' + 'List')
		expect(source).not.toContain('telegramChatGroupFilters(')
		expect(source).toContain('telegram-runtime-panel')
		expect(source).not.toContain('telegram-page')
	})
})
