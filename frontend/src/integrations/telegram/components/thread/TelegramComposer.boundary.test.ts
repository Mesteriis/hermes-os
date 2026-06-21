import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('TelegramComposer dry-run boundary', () => {
  it('embeds the dry-run panel and exposes selected chat scope props for it', () => {
    const source = readFileSync(new URL('./TelegramComposer.vue', import.meta.url), 'utf8')

    expect(source).toContain('TelegramSendDryRunPanel')
    expect(source).toContain('TelegramUploadQueueStatus')
    expect(source).toContain('selectedAccountId: string | null')
    expect(source).toContain('selectedProviderChatId: string | null')
    expect(source).toContain('capabilities?: TelegramCapabilitiesResponse | null')
    expect(source).toContain(':accountId="selectedAccountId"')
    expect(source).toContain(':providerChatId="selectedProviderChatId"')
    expect(source).toContain(':selectedAccountId="selectedAccountId"')
    expect(source).toContain(':selectedProviderChatId="selectedProviderChatId"')
  })

  it('derives disabled media and voice affordance reasons from the capability contract', () => {
    const source = readFileSync(new URL('./TelegramComposer.vue', import.meta.url), 'utf8')

    expect(source).toContain('telegramComposerMediaCapabilityHint')
    expect(source).toContain('telegramComposerVoiceCapabilityHint')
    expect(source).toContain('mediaCapabilityHint.summary')
    expect(source).toContain('voiceCapabilityHint.summary')
    expect(source).not.toContain('Attachment upload is not available in this slice')
    expect(source).not.toContain('Voice messages require media runtime')
  })

  it('exposes media file selection as an event without component-level API calls', () => {
    const source = readFileSync(new URL('./TelegramComposer.vue', import.meta.url), 'utf8')

    expect(source).toContain('uploadMedia: [file: File]')
    expect(source).toContain('ref="fileInput"')
    expect(source).toContain('@change="onMediaFileSelected"')
    expect(source).toContain("emit('uploadMedia', file)")
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toContain('/api/v1/integrations/telegram/media/upload')
  })
})
