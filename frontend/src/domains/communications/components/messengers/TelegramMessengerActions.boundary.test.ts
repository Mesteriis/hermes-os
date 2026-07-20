import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function source(path: string): string {
  return readFileSync(new URL(path, import.meta.url), 'utf8')
}

describe('Telegram messenger action boundary', () => {
  it('keeps the disabled Telegram runtime out of recovery app composition and host access', () => {
    const appRoot = source('../../../../app/layout/AppLayoutRoot.vue')
    const tauriShell = source('../../../../../src-tauri/src/lib.rs')

    expect(appRoot).not.toContain('useTelegramConversationRuntimeActions')
    expect(appRoot).not.toContain('TelegramRuntimePanel')
    expect(appRoot).not.toContain('telegram-runtime-action-runner')
    expect(tauriShell).not.toContain('telegram')
  })
})
