#!/usr/bin/env node

import fs from 'node:fs'
import http from 'node:http'
import https from 'node:https'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const strictEnv = process.env.HERMES_LIVE_SMOKE_STRICT_ENV === '1'
const probeRuntimeApi = process.env.HERMES_WHATSAPP_RUNTIME_API_PROBE === '1'
const runtimeApiBaseUrl =
  process.env.HERMES_WHATSAPP_RUNTIME_API_BASE_URL ?? 'http://127.0.0.1:8080'
const runtimeApiSecret = process.env.HERMES_LOCAL_API_SECRET?.trim() ?? ''
const smokeAccountId = process.env.HERMES_WHATSAPP_SMOKE_ACCOUNT_ID?.trim() ?? ''
const expectedProviderShape = process.env.HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim() ?? ''

const checks = []

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function pass(id, evidence) {
  checks.push({ id, status: 'pass', evidence })
}

function fail(id, evidence) {
  checks.push({ id, status: 'fail', evidence })
}

function requireContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const missing = needles.filter((needle) => !text.includes(needle))
  if (missing.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} contains ${needle}`))
  } else {
    fail(
      id,
      missing.map((needle) => `${relativePath} missing ${needle}`)
    )
  }
}

function requireNotContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const present = needles.filter((needle) => text.includes(needle))
  if (present.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} does not contain ${needle}`))
  } else {
    fail(
      id,
      present.map((needle) => `${relativePath} still contains ${needle}`)
    )
  }
}

function requireEnvWhenStrict(id, envNames) {
  if (!strictEnv) {
    pass(id, ['strict env checks disabled; set HERMES_LIVE_SMOKE_STRICT_ENV=1 for manual smoke'])
    return
  }
  const missing = envNames.filter((name) => !process.env[name]?.trim())
  if (missing.length === 0) {
    pass(id, envNames.map((name) => `${name} is set`))
  } else {
    fail(id, missing.map((name) => `${name} is required for strict live smoke readiness`))
  }
}

function parseJson(value) {
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

function requestRuntimeApi(method, pathname, headers = {}, body = '') {
  return new Promise((resolve, reject) => {
    const url = new URL(pathname, runtimeApiBaseUrl)
    const transport = url.protocol === 'https:' ? https : http
    const requestBody = Buffer.from(body)
    const req = transport.request(
      url,
      {
        method,
        headers: {
          ...headers,
          ...(requestBody.length > 0 ? { 'Content-Length': String(requestBody.length) } : {}),
        },
        timeout: 5_000,
      },
      (res) => {
        const chunks = []
        res.on('data', (chunk) => chunks.push(chunk))
        res.on('end', () => {
          resolve({
            statusCode: res.statusCode ?? 0,
            headers: res.headers,
            body: Buffer.concat(chunks).toString('utf8'),
          })
        })
      }
    )
    req.on('timeout', () => {
      req.destroy(new Error(`request timed out: ${method} ${url.href}`))
    })
    req.on('error', reject)
    if (requestBody.length > 0) {
      req.write(requestBody)
    }
    req.end()
  })
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function providerShapeErrors(value) {
  if (!expectedProviderShape) {
    return []
  }
  return value === expectedProviderShape
    ? []
    : [`provider_shape ${value ?? '<missing>'} did not match ${expectedProviderShape}`]
}

function assertNoSecretLeaks(id, responses) {
  const leakPatterns = [
    /session_blob/i,
    /session_material/i,
    /cookie_value/i,
    /raw_secret/i,
    /access_token_value/i,
    /app_secret_value/i,
    /verify_token_value/i,
    /refresh_token_value/i,
    /browser_profile_secret/i,
    /"authorization"\s*:/i,
    /"cookie"\s*:/i,
    /"qr_code"\s*:\s*"/i,
    /"pair_code"\s*:\s*"/i,
    /"media_key"\s*:\s*"/i,
    /"direct_path"\s*:\s*"/i,
    /"static_url"\s*:\s*"/i,
  ]
  const leaked = []
  for (const response of responses) {
    for (const pattern of leakPatterns) {
      if (pattern.test(response.body)) {
        leaked.push(`${response.id} matched ${pattern.source}`)
      }
    }
  }
  if (leaked.length === 0) {
    pass(id, ['runtime API responses contain no raw secret/session/media-ref payload markers'])
  } else {
    fail(id, leaked)
  }
}

async function probeJsonEndpoint(id, pathname, validate, responses) {
  const response = await requestRuntimeApi('GET', pathname, {
    Accept: 'application/json',
    'X-Hermes-Secret': runtimeApiSecret,
  })
  responses.push({ id, body: response.body })

  if (response.statusCode < 200 || response.statusCode >= 300) {
    fail(id, [`GET ${pathname} returned HTTP ${response.statusCode} (${response.body.length} bytes)`])
    return
  }

  const json = parseJson(response.body)
  if (!isPlainObject(json)) {
    fail(id, [`GET ${pathname} returned non-object JSON (${response.body.length} bytes)`])
    return
  }

  const errors = validate(json)
  if (errors.length === 0) {
    pass(id, [`GET ${pathname} returned the expected sanitized contract`])
  } else {
    fail(id, errors)
  }
}

async function probeRuntimeApiEndpoints() {
  if (!probeRuntimeApi) {
    pass('runtime_api_probe', [
      'runtime API probe disabled; set HERMES_WHATSAPP_RUNTIME_API_PROBE=1 with HERMES_LOCAL_API_SECRET and HERMES_WHATSAPP_SMOKE_ACCOUNT_ID to probe a running Hermes backend',
    ])
    return
  }

  const missing = []
  if (!runtimeApiSecret) {
    missing.push('HERMES_LOCAL_API_SECRET')
  }
  if (!smokeAccountId) {
    missing.push('HERMES_WHATSAPP_SMOKE_ACCOUNT_ID')
  }
  if (missing.length > 0) {
    fail('runtime_api_probe_configuration', missing.map((name) => `${name} is required for runtime API probe`))
    return
  }

  pass('runtime_api_probe_configuration', [
    `probing ${runtimeApiBaseUrl}`,
    `account id is set (${smokeAccountId.length} chars)`,
    expectedProviderShape
      ? `expected provider shape ${expectedProviderShape}`
      : 'provider-shape assertion disabled; set HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE to enable it',
  ])

  const responses = []
  const encodedAccountId = encodeURIComponent(smokeAccountId)
  const encodedStatusAccountId = encodeURIComponent(smokeAccountId)

  try {
    await probeJsonEndpoint(
      'runtime_api_global_capabilities',
      '/api/v1/integrations/whatsapp/capabilities',
      (json) => [
        ...(typeof json.version === 'string' ? [] : ['version must be a string']),
        ...(typeof json.runtime_mode === 'string' ? [] : ['runtime_mode must be a string']),
        ...(Array.isArray(json.provider_shapes) ? [] : ['provider_shapes must be an array']),
        ...(json.account_scope === null ? [] : ['global capabilities account_scope must be null']),
        ...(Array.isArray(json.capabilities) ? [] : ['capabilities must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_account_capabilities',
      `/api/v1/integrations/whatsapp/accounts/${encodedAccountId}/capabilities`,
      (json) => [
        ...(isPlainObject(json.account_scope) ? [] : ['account_scope must be an object']),
        ...(json.account_scope?.account_id === smokeAccountId
          ? []
          : ['account_scope.account_id must match HERMES_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.account_scope?.provider_shape === 'string'
          ? []
          : ['account_scope.provider_shape must be a string']),
        ...providerShapeErrors(json.account_scope?.provider_shape),
        ...(Array.isArray(json.capabilities) ? [] : ['capabilities must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_status',
      `/api/v1/integrations/whatsapp/runtime/status?account_id=${encodedStatusAccountId}`,
      (json) => [
        ...(json.account_id === smokeAccountId ? [] : ['account_id must match HERMES_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.provider_shape === 'string' ? [] : ['provider_shape must be a string']),
        ...providerShapeErrors(json.provider_shape),
        ...(typeof json.runtime_kind === 'string' ? [] : ['runtime_kind must be a string']),
        ...(typeof json.status === 'string' ? [] : ['status must be a string']),
        ...(typeof json.session_restore_available === 'boolean'
          ? []
          : ['session_restore_available must be a boolean']),
        ...(Array.isArray(json.runtime_blockers) ? [] : ['runtime_blockers must be an array']),
      ],
      responses
    )

    await probeJsonEndpoint(
      'runtime_api_health',
      `/api/v1/integrations/whatsapp/runtime/health?account_id=${encodedStatusAccountId}`,
      (json) => [
        ...(json.account_id === smokeAccountId ? [] : ['account_id must match HERMES_WHATSAPP_SMOKE_ACCOUNT_ID']),
        ...(typeof json.provider_shape === 'string' ? [] : ['provider_shape must be a string']),
        ...providerShapeErrors(json.provider_shape),
        ...(typeof json.runtime_kind === 'string' ? [] : ['runtime_kind must be a string']),
        ...(typeof json.status === 'string' ? [] : ['status must be a string']),
        ...(typeof json.healthy === 'boolean' ? [] : ['healthy must be a boolean']),
        ...(isPlainObject(json.checks) ? [] : ['checks must be an object']),
        ...(typeof json.checked_at === 'string' ? [] : ['checked_at must be a string']),
      ],
      responses
    )

    assertNoSecretLeaks('runtime_api_probe_no_secret_leaks', responses)
  } catch (error) {
    fail('runtime_api_probe_request', [
      error instanceof Error ? error.message : 'runtime API probe failed with an unknown error',
    ])
  }
}

requireContains('webview_runtime_event_relay_dispatch', 'frontend/src-tauri/src/whatsapp_companion.rs', [
  'RUNTIME_EVENTS_BRIDGE_PATH',
  '"/api/v1/integrations/whatsapp/runtime-bridge/runtime-events"',
  'runtime_bridge_runtime_event_payload',
  'dispatch_runtime_bridge_runtime_event',
  'X-Hermes-Secret',
  'is_allowed_local_backend_url',
  'runtime_event_evidence_only_until_richer_typed_payload',
  'provider_observed_event_reconciliation_required',
])

requireContains('webview_remote_capability_is_narrow', 'frontend/src-tauri/capabilities/whatsapp-companion-relay.json', [
  '"local": false',
  'https://web.whatsapp.com',
  'whatsapp-companion-*',
  'allow-whatsapp-web-companion-relay-observation',
])

requireNotContains('webview_remote_capability_no_broad_core', 'frontend/src-tauri/capabilities/whatsapp-companion-relay.json', [
  'core:default',
])

requireNotContains('main_capability_does_not_allow_remote_relay', 'frontend/src-tauri/capabilities/default.json', [
  'allow-whatsapp-web-companion-relay-observation',
])

requireContains('webview_backend_health_contract_dispatch_state', 'backend/src/integrations/whatsapp/runtime/web_companion.rs', [
  'contract_injected_relay_dispatch_available',
  'tauri_allowlisted_companion_runtime_bridge_dispatch',
  'runtime_events_bridge_wired_smoke_pending',
  'NewWhatsappWebRuntimeEvent',
  'X-Hermes-Secret_from_tauri_process_env_only',
  'typed_projection',
  'manual_live_smoke_required',
])

requireNotContains('webview_backend_health_no_old_dispatch_blocker', 'backend/src/integrations/whatsapp/runtime/web_companion.rs', [
  'whatsapp_webview_backend_dispatch_not_implemented',
  'blocked_until_backend_dispatch_is_wired_and_live_smoked',
  'contract_injected_relay_preflight_available',
])

requireContains('signal_hub_static_guard_covers_webview_dispatch', 'backend/tests/whatsapp_signal_hub.rs', [
  'dispatch_runtime_bridge_runtime_event',
  'RUNTIME_EVENTS_BRIDGE_PATH',
  'is_allowed_local_backend_url',
  'runtime_event_evidence_only_until_richer_typed_payload',
  'dispatched_to_backend_runtime_bridge_runtime_event',
])

requireContains('runtime_panel_exposes_webview_companion_action', 'frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue', [
  "import { openWhatsappWebCompanion } from '../api/whatsappCompanion'",
  'async function openVisibleWebCompanion()',
  "selectedRuntimeProviderShape.value === 'whatsapp_web_companion'",
  'openWhatsappWebCompanion(accountId)',
  'Open Companion',
  'companionOpenManifest.event_extractor.relay_channel',
])

requireNotContains('runtime_panel_companion_action_avoids_backend_http', 'frontend/src/integrations/whatsapp/views/WhatsAppRuntimePanel.vue', [
  'window.fetch',
  'globalThis.fetch',
  '/api/v1/integrations/whatsapp/runtime-bridge',
  'ApiClient',
])

requireContains('manual_smoke_checklist_has_exit_criteria', 'docs/integrations/whatsapp/live-smoke-checklist.md', [
  'Open Companion',
  'contract_injected_relay_dispatch_available',
  'tauri_allowlisted_companion_runtime_bridge_dispatch',
  'runtime_events_bridge_wired_smoke_pending',
  '.local/whatsapp/live-smoke-evidence.json',
  'make whatsapp-native-md-sdk-gap-readiness',
  'make whatsapp-live-smoke-evidence',
  'Each passed gate must also include concrete sanitized `evidence_refs`',
  '`command:` plus observed event refs',
  '`vault_binding:` for session/credential binding',
  'session restore works from vault-bound state',
  'writes reconcile through observed provider evidence',
  'no hidden or headless runtime behavior was required',
])

requireContains('manual_smoke_evidence_validator_contract', 'scripts/whatsapp-live-smoke-evidence.mjs', [
  'defaultEvidencePath = \'.local/whatsapp/live-smoke-evidence.json\'',
  'commonGateIds',
  'personalGateIds',
  'businessCloudGateIds',
  'allowedEvidenceRefPrefixes',
  'requiredEvidenceRefPrefixGroups',
  'evidence_refs',
  'account_fingerprint must be sha256:<64 hex chars>',
  'no_hidden_or_headless_runtime',
  'evidence.${gateId}.status must be passed',
  'weak_reconciliation_refs_fail',
  'placeholder_refs_fail',
  'secretLikePatterns',
])

requireContains('manual_smoke_evidence_make_target', 'Makefile', [
  'whatsapp-live-smoke-evidence:',
  'node scripts/whatsapp-live-smoke-evidence.mjs',
])

requireContains('manual_smoke_evidence_collector_contract', 'scripts/whatsapp-live-smoke-collect-evidence.mjs', [
  'defaultObservationsPath = \'.local/whatsapp/live-smoke-observations.json\'',
  'whatsapp-live-smoke-evidence.mjs',
  '--observations-template',
  'HERMES_WHATSAPP_SMOKE_ACCOUNT_ID',
  'sha256Fingerprint',
  'assertNoSecretLikeContent',
  'mergeEvidence(template, observations)',
  'Gates without operator-provided',
])

requireContains('manual_smoke_evidence_collector_docs', 'docs/integrations/whatsapp/live-smoke-checklist.md', [
  'make whatsapp-live-smoke-collect-evidence',
  '.local/whatsapp/live-smoke-observations.json',
  'normalizer, not a bypass',
  'Gates without operator-provided sanitized',
])

requireContains('manual_smoke_evidence_collector_make_target', 'Makefile', [
  'whatsapp-live-smoke-collect-evidence:',
  'node scripts/whatsapp-live-smoke-collect-evidence.mjs',
])

requireContains('domain_closure_audit_contract', 'scripts/whatsapp-domain-closure-audit.mjs', [
  'requiredEvidenceShapes',
  'whatsapp_native_md',
  'whatsapp_web_companion',
  'whatsapp_business_cloud',
  'adr_0101_acceptance_scope_keeps_live_blocked',
  'validateEvidenceFile(filePath)',
  'scripts/whatsapp-live-smoke-evidence.mjs',
  'native_md_unsupported_commands_remaining',
  'adr_0101_accepted',
  'docs_status_does_not_overclaim_closure',
])

requireContains('domain_closure_audit_make_targets', 'Makefile', [
  'whatsapp-domain-closure-audit:',
  'node scripts/whatsapp-domain-closure-audit.mjs',
  'whatsapp-domain-closure-gate:',
  'node scripts/whatsapp-domain-closure-audit.mjs --require-closed',
])

requireContains('native_md_sdk_gap_verifier_contract', 'scripts/whatsapp-native-md-sdk-gap-readiness.mjs', [
  "const waRsVersion = '0.2.0'",
  'verifyRustAndCrateUpgradeContext()',
  'native_md_rust_baseline_context',
  'native_md_crates_io_probe',
  'native_md_upgrade_requires_provider_api_not_toolchain_only',
  'requiredApis',
  'unsupportedExpectations',
  'verifyLowLevelGapEvidence(root)',
  'custom_iq_api_exists_but_is_low_level',
  'no_public_outgoing_appstate_encoder',
  'no_join_by_invite_iq_spec',
  'waRsSourceRoot()',
  'publicFunctionNames(root)',
  'native_md_wa_rs_sdk_command_gap_health',
  'never_completed_without_provider_observed_event',
])

requireContains('native_md_sdk_gap_make_target', 'Makefile', [
  'whatsapp-native-md-sdk-gap-readiness:',
  'node scripts/whatsapp-native-md-sdk-gap-readiness.mjs',
])

requireContains('status_tracks_remaining_live_gates', 'docs/integrations/whatsapp/status.md', [
  'IMPLEMENTED CHECKPOINTS = 67',
  'DOMAIN CLOSURE          = not achieved',
  'WebView live smoke',
  'web companion runtime-panel action',
  'business cloud edge smoke readiness harness',
  'runtime API smoke probe',
  'manual smoke evidence validator',
  'remaining native SDK command gap verifier',
  'domain closure audit gate',
  'ADR-0101 accepted runtime decision',
  'native low-level SDK gap evidence',
  'native forward text reemit submission',
  'strict live-smoke evidence references',
  'native Rust/wa-rs upgrade path verifier',
  'live-smoke evidence collector',
  'Business Cloud public exposure/smoke',
])

requireEnvWhenStrict('strict_manual_smoke_env', [
  'HERMES_LOCAL_API_SECRET',
  'HERMES_WHATSAPP_SMOKE_ACCOUNT_ID',
])

await probeRuntimeApiEndpoints()

const failed = checks.filter((check) => check.status === 'fail')
const result = {
  ok: failed.length === 0,
  strict_env: strictEnv,
  runtime_api_probe: probeRuntimeApi,
  generated_at: new Date().toISOString(),
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (failed.length > 0) {
  process.exitCode = 1
}
