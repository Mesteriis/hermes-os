import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('TelegramRuntimePanel realtime boundary', () => {
	it('preserves telegram runtime orchestration after removing the legacy runtime panel Vue layer', () => {
		const surfaceSource = readFileSync(
			new URL('../queries/useTelegramRuntimePanelSurface.ts', import.meta.url),
			'utf8'
		)
		const runtimeQuerySource = readFileSync(
			new URL('../queries/useTelegramRuntimeQuery.ts', import.meta.url),
			'utf8'
		)
		const querySource = readFileSync(
			new URL('../queries/useTelegramQuery.ts', import.meta.url),
			'utf8'
		)

		expect(existsSync(new URL('./TelegramRuntimePanel.vue', import.meta.url))).toBe(false)
		expect(surfaceSource).toContain('useRealtimeStatusStore()')
		expect(surfaceSource).toContain('useTelegramAccountsQuery()')
		expect(surfaceSource).toContain('useTelegramCapabilitiesQuery()')
		expect(surfaceSource).toContain('useTelegramRuntimeStatusQuery(')
		expect(surfaceSource).toContain('useStopTelegramRuntimeMutation()')
		expect(surfaceSource).toContain('useStartTelegramRuntimeMutation()')
		expect(surfaceSource).toContain('useRestartTelegramRuntimeMutation()')
		expect(surfaceSource).toContain("async function setTelegramRuntime(action: 'start' | 'stop' | 'restart')")
		expect(surfaceSource).toContain('selectedAccountIdState')
		expect(surfaceSource).toContain('await Promise.all([')
		expect(surfaceSource).toContain('runtimeStatusQuery.refetch()')
		expect(surfaceSource).not.toContain('.vue')
		expect(surfaceSource).not.toContain('useTelegramMessagesQuery(')
		expect(surfaceSource).not.toContain('useTelegramMessageSearchQuery(')
		expect(surfaceSource).not.toContain('useTelegramMediaSearchQuery(')

		expect(runtimeQuerySource).toContain('fetchTelegramRuntimeStatus')
		expect(runtimeQuerySource).toContain('useStartTelegramRuntimeMutation')
		expect(runtimeQuerySource).toContain('useStopTelegramRuntimeMutation')
		expect(runtimeQuerySource).toContain('useRestartTelegramRuntimeMutation')
		expect(querySource).toContain('useTelegramAccountsQuery')
		expect(querySource).toContain('useTelegramCapabilitiesQuery')
	})
})
