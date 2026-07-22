import { afterEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import {
  getWhatsappWebCompanionManifest,
  startHiddenWhatsappWebview,
  relayWhatsappWebCompanionObservation,
} from './whatsappCompanion'

describe('whatsapp WebView companion Tauri bridge', () => {
  afterEach(() => {
    invokeMock.mockReset()
    vi.unstubAllGlobals()
  })

  it('starts the hidden WebView through Tauri invoke, not backend HTTP', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: true }))

    const result = await startHiddenWhatsappWebview(' wa-live-1 ')

    expect(result.opened_window).toBe(true)
    expect(result.provider_shape).toBe('whatsapp_web_companion')
    expect(invokeMock).toHaveBeenCalledWith('start_hidden_whatsapp_webview', {
      request: { account_id: 'wa-live-1' },
    })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('loads the sanitized manifest without exposing secret-bearing fields', async () => {
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: false }))

    const result = await getWhatsappWebCompanionManifest('wa-live-1')

    expect(invokeMock).toHaveBeenCalledWith('whatsapp_web_companion_manifest', {
      request: { account_id: 'wa-live-1' },
    })
    expect(result.target_url).toBe('https://web.whatsapp.com/')
    expect(result.bridge_routes.authorized_session_path).toBe(
      'whatsapp.client://observation/sessions/authorized'
    )
    expect(result.command_channel.completion_rule).toBe(
      'provider_observed_event_reconciliation_required'
    )
    expect(result.event_extractor.state).toBe(
      'contract_injected_relay_dispatch_available'
    )
    expect(result.event_extractor.origin_guard).toBe('https://web.whatsapp.com')
    expect(result.event_extractor.relay_channel).toBe(
      'tauri_versioned_whatsapp_host_bridge_unix_socket'
    )
    expect(result.event_extractor.runtime_bridge_dispatch).toBe(
      'versioned_whatsapp_runtime_socket_wired'
    )
    expect(result.event_extractor.forbidden_reads).toContain('message_bodies')
    expect(result.event_extractor.forbidden_reads).toContain('media_bytes')
    expect(result.secret_policy.cookies).toBe('not_read_or_returned_by_tauri_command')
    expect(JSON.stringify(result)).not.toContain('cookie_value')
    expect(JSON.stringify(result)).not.toContain('session_blob')
  })

  it('relays sanitized companion observations only through the Tauri relay command', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    invokeMock.mockResolvedValueOnce({
      account_id: 'wa-live-1',
      provider_shape: 'whatsapp_web_companion',
      runtime_kind: 'webview_companion',
      window_label: 'whatsapp-companion-wa-live-1',
      event_family: 'message',
      provider_event_id: 'provider-event-1',
      observed_at: '1782504000',
      target_runtime_bridge_path:
        'whatsapp.client://observation/runtime-events',
      typed_runtime_bridge_path:
        'whatsapp.client://observation/messages',
      relay_state: 'accepted_by_whatsapp_runtime',
      relay_channel: 'tauri_versioned_whatsapp_host_bridge_unix_socket',
      sanitized_metadata: { provider_chat_id: 'chat-1' },
      runtime_event_kind: 'webview_companion.message.observed',
      import_batch_id: 'whatsapp-webview-companion:wa-live-1:provider-event-1',
      runtime_bridge_status: 'accepted',
      event_flow:
        'hidden_webview_companion -> tauri_versioned_host_bridge -> whatsapp_runtime -> provider_observation_projection',
      completion_rule: 'provider_observed_event_reconciliation_required',
    })

    const result = await relayWhatsappWebCompanionObservation(' wa-live-1 ', {
      event_family: 'message',
      provider_event_id: 'provider-event-1',
      observed_at: '1782504000',
      metadata: {
        provider_chat_id: 'chat-1',
      },
    })

    expect(result.relay_state).toBe('accepted_by_whatsapp_runtime')
    expect(result.target_runtime_bridge_path).toBe(
      'whatsapp.client://observation/runtime-events'
    )
    expect(result.typed_runtime_bridge_path).toBe(
      'whatsapp.client://observation/messages'
    )
    expect(result.runtime_event_kind).toBe('webview_companion.message.observed')
    expect(result.runtime_bridge_status).toBe('accepted')
    expect(invokeMock).toHaveBeenCalledWith('whatsapp_web_companion_relay_observation', {
      request: {
        account_id: 'wa-live-1',
        event_family: 'message',
        provider_event_id: 'provider-event-1',
        observed_at: '1782504000',
        metadata: {
          provider_chat_id: 'chat-1',
        },
      },
    })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('rejects empty account ids before invoking Tauri', async () => {
    await expect(startHiddenWhatsappWebview(' ')).rejects.toThrow(
      'account_id is required for WhatsApp Web companion'
    )

    expect(invokeMock).not.toHaveBeenCalled()
  })
})

function companionManifest(overrides: Partial<{ opened_window: boolean }>) {
  return {
    account_id: 'wa-live-1',
    provider_shape: 'whatsapp_web_companion',
    runtime_kind: 'webview_companion',
    driver_id: 'tauri_hidden_webview_companion',
    window_label: 'whatsapp-companion-wa-live-1',
    target_url: 'https://web.whatsapp.com/',
    opened_window: overrides.opened_window ?? false,
    reused_existing_window: false,
    owner_visible: false,
    hidden_headless_mode: 'required_tauri_webview_not_headless_browser',
    tauri_ipc_available_to_companion_window: false,
    event_flow:
      'hidden_webview_companion -> protected_runtime_bridge -> raw_evidence -> signal_hub_accepted -> projection_reconciliation',
    event_extractor: {
      state: 'contract_injected_relay_dispatch_available',
      relay_command: 'whatsapp_web_companion_relay_observation',
      initialization_script: 'installed_on_hidden_companion_webview',
      script_scope: 'main_frame_only',
      origin_guard: 'https://web.whatsapp.com',
      navigation_guard: 'https://web.whatsapp.com_only',
      relay_channel: 'tauri_versioned_whatsapp_host_bridge_unix_socket',
      runtime_bridge_dispatch: 'versioned_whatsapp_runtime_socket_wired',
      allowed_observations: [
        'runtime_lifecycle_metadata',
        'message_identity_metadata',
        'media_metadata_without_bytes',
      ],
      forbidden_reads: [
        'cookies',
        'web_storage',
        'indexed_db',
        'browser_profile_secrets',
        'session_material',
        'message_bodies',
        'media_bytes',
      ],
      next_gate: 'manual_live_smoke_before_public_availability',
    },
    bridge_routes: {
      authorized_session_path:
        'whatsapp.client://observation/sessions/authorized',
      runtime_event_path: 'whatsapp.client://observation/runtime-events',
      sync_lifecycle_path: 'whatsapp.client://observation/sync-lifecycle',
      message_paths: ['whatsapp.client://observation/messages'],
      conversation_paths: ['whatsapp.client://observation/dialogs'],
      media_paths: ['whatsapp.client://observation/media'],
    },
    command_channel: {
      kind: 'durable_outbox',
      claim_path: 'whatsapp.client://observation/commands/claim',
      failure_path:
        'whatsapp.client://observation/commands/{command_id}/failed',
      completion_rule: 'provider_observed_event_reconciliation_required',
    },
    secret_policy: {
      session_material: 'host_vault_only_via_authorized_session_bridge',
      cookies: 'not_read_or_returned_by_tauri_command',
      browser_profile_secrets: 'not_read_or_returned_by_tauri_command',
      qr_pair_code_artifacts: 'never_returned_by_hidden_webview_runtime',
      message_bodies: 'excluded_from_manifest_and_health',
      media_bytes: 'local_blob_storage_only_not_manifest_or_postgres',
      postgres_storage: 'metadata_bindings_only_no_session_cookie_or_profile_secret',
    },
    remaining_blockers: [
      'whatsapp_webview_runtime_panel_action_not_implemented',
      'whatsapp_webview_live_smoke_required',
    ],
  }
}
