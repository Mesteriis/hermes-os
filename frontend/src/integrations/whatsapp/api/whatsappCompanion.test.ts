import { afterEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import {
  getWhatsappWebCompanionManifest,
  hideWhatsappWebCompanion,
  openWhatsappWebCompanionForPairing,
  startHiddenWhatsappWebview,
} from './whatsappCompanion'

describe('WhatsApp WebView host executor Tauri bridge', () => {
  afterEach(() => {
    invokeMock.mockReset()
    vi.unstubAllGlobals()
  })

  it('starts the hidden host WebView through Tauri invoke, not backend HTTP', async () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: true }))

    const result = await startHiddenWhatsappWebview(' wa-live-1 ')

    expect(result.opened_window).toBe(true)
    expect(invokeMock).toHaveBeenCalledWith('start_hidden_whatsapp_webview', {
      request: { account_id: 'wa-live-1' },
    })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('opens pairing only through the owner-controlled host command', async () => {
    invokeMock.mockResolvedValueOnce(companionManifest({ owner_visible: true }))

    const result = await openWhatsappWebCompanionForPairing('wa-live-1')

    expect(result.owner_visible).toBe(true)
    expect(invokeMock).toHaveBeenCalledWith('open_whatsapp_web_companion', {
      request: { account_id: 'wa-live-1' },
    })
  })

  it('exposes the lifecycle-only host manifest without provider data routes', async () => {
    invokeMock.mockResolvedValueOnce(companionManifest({ opened_window: false }))

    const result = await getWhatsappWebCompanionManifest('wa-live-1')

    expect(invokeMock).toHaveBeenCalledWith('whatsapp_web_companion_manifest', {
      request: { account_id: 'wa-live-1' },
    })
    expect(result.event_extractor.state).toBe('runtime_lifecycle_relay_only_no_provider_data_extractor')
    expect(result.bridge_routes.runtime_event_path).toBe('exact_active_route_runtime_lifecycle_only')
    expect(result.command_channel.kind).toBe('not_available')
    expect(result.event_extractor.forbidden_reads).toContain('message_bodies')
    expect(result.event_extractor.forbidden_reads).toContain('media_bytes')
    expect(JSON.stringify(result)).not.toContain('cookie_value')
    expect(JSON.stringify(result)).not.toContain('session_blob')
  })

  it('hides a paired WebView through Tauri invoke', async () => {
    invokeMock.mockResolvedValueOnce(companionManifest({ owner_visible: false }))

    const result = await hideWhatsappWebCompanion('wa-live-1')

    expect(result.owner_visible).toBe(false)
    expect(invokeMock).toHaveBeenCalledWith('hide_whatsapp_web_companion', {
      request: { account_id: 'wa-live-1' },
    })
  })
})

function companionManifest(overrides: Partial<{ opened_window: boolean; owner_visible: boolean }>) {
  return {
    account_id: 'wa-live-1',
    provider_shape: 'whatsapp_web_companion',
    runtime_kind: 'webview_companion',
    driver_id: 'tauri_host_only_whatsapp_webview',
    window_label: 'whatsapp-companion-wa-live-1',
    target_url: 'https://web.whatsapp.com/',
    opened_window: overrides.opened_window ?? false,
    reused_existing_window: false,
    owner_visible: overrides.owner_visible ?? false,
    hidden_headless_mode: 'hidden_tauri_webview_after_explicit_owner_pairing',
    tauri_ipc_available_to_companion_window: true,
    event_flow: 'explicit_owner_pairing -> active_exact_host_route -> sanitized_runtime_lifecycle_observation',
    event_extractor: {
      state: 'runtime_lifecycle_relay_only_no_provider_data_extractor',
      initialization_script: 'installed_on_hidden_companion_webview',
      script_scope: 'main_frame_only',
      origin_guard: 'https://web.whatsapp.com',
      navigation_guard: 'https://web.whatsapp.com_only',
      relay_channel: 'whatsapp_web_companion_relay_runtime_state_without_payload',
      runtime_bridge_dispatch: 'exact_active_route_native_typed_client_request',
      allowed_observations: ['host_route_attached', 'webview_loaded'],
      forbidden_reads: ['cookies', 'message_bodies', 'media_bytes'],
      next_gate: 'provider_dom_metadata_extractor_with_live_smoke',
    },
    bridge_routes: {
      authorized_session_path: 'not_available',
      runtime_event_path: 'exact_active_route_runtime_lifecycle_only',
      sync_lifecycle_path: 'not_available',
      message_paths: [],
      conversation_paths: [],
      media_paths: [],
    },
    command_channel: {
      kind: 'not_available',
      claim_path: 'not_available',
      failure_path: 'not_available',
      result_path: 'not_available',
      completion_rule: 'provider_commands_require_admitted_whatsapp_runtime',
    },
    secret_policy: {
      session_material: 'owned_by_os_managed_webview_profile',
      cookies: 'not_read_or_returned_by_tauri',
      browser_profile_secrets: 'not_read_or_returned_by_tauri',
      qr_pair_code_artifacts: 'visible_only_in_owner_controlled_webview',
      message_bodies: 'not_read_by_host_executor',
      media_bytes: 'not_read_by_host_executor',
      postgres_storage: 'not_used_by_host_executor',
    },
    remaining_blockers: [
      'provider_dom_metadata_extractor_not_implemented',
      'provider_command_executor_not_implemented',
      'manual_live_pairing_smoke_required',
    ],
  }
}
