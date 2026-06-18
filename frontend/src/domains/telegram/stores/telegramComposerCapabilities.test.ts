import { describe, expect, it } from 'vitest'
import {
  telegramComposerMediaCapabilityHint,
  telegramComposerVoiceCapabilityHint
} from './telegramComposerCapabilities'
import type { TelegramCapabilitiesResponse } from '../types/telegram'

function capabilitiesFixture(): TelegramCapabilitiesResponse {
  return {
    version: '2026-06-17',
    runtime_mode: 'fixture',
    account_scope: null,
    telegram_app_credentials_configured: false,
    tdjson_runtime_available: false,
    qr_login_ready: false,
    bot_runtime_available: false,
    planned_features: [],
    unsupported_features: [],
    capabilities: [
      {
        operation: 'messages.send_media',
        category: 'message_write',
        status: 'blocked',
        action_class: 'provider_write',
        reason: 'Media upload/send requires durable outbox model and attachment upload pipeline.',
        confirmation_required: true,
        closure_gate: true
      },
      {
        operation: 'voice.record_send',
        category: 'voice_calls',
        status: 'blocked',
        action_class: 'provider_write',
        reason: 'Voice recording/send requires desktop media permission boundary.',
        confirmation_required: true,
        closure_gate: true
      }
    ]
  }
}

describe('telegramComposerCapabilities', () => {
  it('projects media and voice composer hints from the selected account capability contract', () => {
    const capabilities = capabilitiesFixture()

    expect(telegramComposerMediaCapabilityHint(capabilities)).toMatchObject({
      operation: 'messages.send_media',
      status: 'blocked',
      reason: 'Media upload/send requires durable outbox model and attachment upload pipeline.'
    })
    expect(telegramComposerVoiceCapabilityHint(capabilities)).toMatchObject({
      operation: 'voice.record_send',
      status: 'blocked',
      reason: 'Voice recording/send requires desktop media permission boundary.'
    })
  })

  it('uses explicit unknown-state fallbacks when the contract is not loaded', () => {
    expect(telegramComposerMediaCapabilityHint(null)).toMatchObject({
      operation: 'messages.send_media',
      status: 'unknown'
    })
    expect(telegramComposerVoiceCapabilityHint(undefined)).toMatchObject({
      operation: 'voice.record_send',
      status: 'unknown'
    })
  })
})
