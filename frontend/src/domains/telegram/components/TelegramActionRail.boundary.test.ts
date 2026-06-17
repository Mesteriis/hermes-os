import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramActionRail runtime controls', () => {
  it('exposes explicit runtime start and stop events without component-level fetches', () => {
    const source = readFileSync(new URL('./TelegramActionRail.vue', import.meta.url), 'utf8')

    expect(source).toContain("'startRuntime': []")
    expect(source).toContain("'stopRuntime': []")
    expect(source).toContain("'restartRuntime': []")
    expect(source).toContain("emit('startRuntime')")
    expect(source).toContain("emit('stopRuntime')")
    expect(source).toContain("emit('restartRuntime')")
    expect(source).toContain("t('Stop Runtime')")
    expect(source).toContain("t('Restart Runtime')")
    expect(source).not.toContain('fetch(')
  })
})
