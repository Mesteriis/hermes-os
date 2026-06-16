import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramPage realtime boundary', () => {
	it('relies on the global realtime bootstrap instead of opening a page-level telegram socket', () => {
		const source = readFileSync(new URL('./TelegramPage.vue', import.meta.url), 'utf8')

		expect(source).not.toContain('createTelegramRealtimeConnection')
		expect(source).not.toContain('realtimeCleanup')
		expect(source).not.toContain('onMounted(() =>')
		expect(source).not.toContain('onUnmounted(() =>')
		expect(source).toContain('useTelegramMessagesQuery(')
		expect(source).toContain('useTelegramMessageSearchQuery({')
		expect(source).toContain('useTelegramMediaSearchQuery({')
		expect(source).toContain('useTelegramFolderFilters(')
		expect(source).toContain('selectedAccountId,')
		expect(source).toContain('useTelegramAccountCapabilitiesQuery(selectedAccountId)')
		expect(source).toContain('useTelegramChatDetailQuery(')
		expect(source).toContain('useTelegramChatMembersQuery(')
		expect(source).toContain('useMarkReadTelegramChatMutation()')
		expect(source).toContain('useMarkUnreadTelegramChatMutation()')
		expect(source).toContain('usePinTelegramMessageMutation()')
		expect(source).toContain(':focusedTelegramMessage="store.focusedTelegramMessage"')
		expect(source).toContain(':selectedTelegramRuntimeStatus="selectedRuntimeStatus ?? null"')
		expect(source).toContain('@togglePinMessage="(message) => void togglePinnedTelegramMessage(message)"')
		expect(source).toContain('@toggleReadChat="void toggleReadTelegramChat()"')
		expect(source).toContain("store.openTelegramInspector('about')")
		expect(source).not.toContain('telegramChatGroupFilters(')
	})
})
