#!/usr/bin/env node

import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const defaultEvidencePath = '.local/whatsapp/live-smoke-evidence.json'
const evidencePath =
  process.env.HERMES_WHATSAPP_LIVE_SMOKE_EVIDENCE ?? defaultEvidencePath

const providerShapes = new Set([
  'whatsapp_web_companion',
  'whatsapp_native_md',
  'whatsapp_business_cloud',
])

const commonGateIds = [
  'preflight.readiness_target',
  'preflight.runtime_api_probe',
  'runtime.boundary_provider_shape',
  'runtime.health_sanitized',
  'event_flow.raw_evidence_append_only',
  'event_flow.accepted_signal_events',
  'event_flow.projection_from_event_spine',
  'event_flow.no_direct_domain_writes',
  'commands.no_completion_without_provider_observed_evidence',
  'media.bytes_local_blob_only',
  'media.scanner_default_not_scanned',
  'media.no_clean_without_scanner',
  'redaction.api_responses',
  'redaction.raw_evidence',
  'redaction.event_payloads',
  'redaction.logs',
  'redaction.frontend_payloads',
]

const personalGateIds = [
  'auth.qr_or_pair_code_login',
  'auth.host_vault_session_binding',
  'auth.restart_restore_without_user_action',
  'auth.session_rotation_or_relink',
  'auth.multi_account_isolation',
  'inbound.private_message',
  'inbound.group_message',
  'inbound.reply_or_quote',
  'inbound.forward',
  'inbound.edit',
  'inbound.delete',
  'inbound.receipt',
  'inbound.reaction',
  'inbound.dialog',
  'inbound.participant',
  'inbound.presence',
  'inbound.call_metadata',
  'inbound.status',
  'inbound.status_view',
  'inbound.status_delete',
  'inbound.media_metadata',
  'inbound.media_download_ref',
  'inbound.sync_lifecycle',
  'outbound.send_text',
  'outbound.reply',
  'outbound.forward',
  'outbound.edit',
  'outbound.delete',
  'outbound.react',
  'outbound.unreact',
  'outbound.media_upload',
  'outbound.media_download',
  'outbound.voice_note',
  'outbound.status_publish',
  'outbound.mark_read',
  'outbound.mark_unread',
  'outbound.archive',
  'outbound.unarchive',
  'outbound.mute',
  'outbound.unmute',
  'outbound.pin',
  'outbound.unpin',
  'outbound.join_group',
  'outbound.leave_group',
  'search.message_search',
  'search.media_search',
  'search.participant_search',
  'search.chat_search',
]

const businessCloudGateIds = [
  'business_cloud.host_vault_access_token_binding',
  'business_cloud.host_vault_app_secret_binding',
  'business_cloud.host_vault_verify_token_binding',
  'business_cloud.edge_proxy_public_only_path',
  'business_cloud.edge_proxy_meta_challenge',
  'business_cloud.edge_proxy_signed_webhook',
  'business_cloud.hermes_api_not_public',
  'business_cloud.inbound_message_webhook',
  'business_cloud.receipt_webhook_reconciliation',
  'business_cloud.send_text',
  'business_cloud.send_template',
  'business_cloud.send_media',
  'business_cloud.send_voice_note',
  'business_cloud.rate_limit_retry_hint',
  'business_cloud.not_personal_whatsapp_substitute',
]

const secretLikePatterns = [
  /session_blob/i,
  /session_material/i,
  /cookie_value/i,
  /raw_secret/i,
  /access_token_value/i,
  /refresh_token_value/i,
  /app_secret_value/i,
  /verify_token_value/i,
  /"authorization"\s*:/i,
  /"cookie"\s*:/i,
  /browser_profile_secret/i,
  /"qr_code"\s*:/i,
  /"pair_code"\s*:/i,
  /"media_key"\s*:/i,
  /"direct_path"\s*:/i,
  /"static_url"\s*:/i,
  /\+\d{7,15}/,
  /\b\d{8,15}@s\.whatsapp\.net\b/i,
]

const allowedEvidenceRefPrefixes = [
  'audit:',
  'blob:',
  'command:',
  'doc:',
  'edge_proxy:',
  'event_log:',
  'log_scan:',
  'projection:',
  'raw_record:',
  'runtime_api:',
  'search:',
  'signal_hub:',
  'storage:',
  'ui:',
  'vault_binding:',
]

function requiredGateIds(providerShape) {
  if (providerShape === 'whatsapp_business_cloud') {
    return [...commonGateIds, ...businessCloudGateIds]
  }
  return [...commonGateIds, ...personalGateIds]
}

function requiredEvidenceRefPrefixGroups(providerShape, gateId) {
  const common = {
    'preflight.readiness_target': [['command:']],
    'preflight.runtime_api_probe': [['runtime_api:']],
    'runtime.boundary_provider_shape': [['runtime_api:']],
    'runtime.health_sanitized': [['runtime_api:']],
    'event_flow.raw_evidence_append_only': [['raw_record:']],
    'event_flow.accepted_signal_events': [['event_log:', 'signal_hub:']],
    'event_flow.projection_from_event_spine': [['projection:']],
    'event_flow.no_direct_domain_writes': [['audit:']],
    'commands.no_completion_without_provider_observed_evidence': [
      ['command:'],
      ['event_log:', 'signal_hub:'],
    ],
    'media.bytes_local_blob_only': [['blob:', 'storage:']],
    'media.scanner_default_not_scanned': [['projection:', 'storage:']],
    'media.no_clean_without_scanner': [['audit:', 'projection:']],
    'redaction.api_responses': [['runtime_api:']],
    'redaction.raw_evidence': [['raw_record:']],
    'redaction.event_payloads': [['event_log:', 'signal_hub:']],
    'redaction.logs': [['log_scan:']],
    'redaction.frontend_payloads': [['ui:']],
  }
  if (common[gateId]) {
    return common[gateId]
  }

  if (gateId.startsWith('auth.')) {
    if (gateId === 'auth.host_vault_session_binding') {
      return [['vault_binding:']]
    }
    if (gateId === 'auth.restart_restore_without_user_action') {
      return [['event_log:', 'runtime_api:']]
    }
    return [['runtime_api:', 'event_log:']]
  }
  if (gateId.startsWith('inbound.')) {
    return [['raw_record:'], ['event_log:', 'signal_hub:']]
  }
  if (gateId.startsWith('outbound.')) {
    if (gateId === 'outbound.media_download') {
      return [['command:'], ['event_log:', 'signal_hub:'], ['blob:', 'storage:']]
    }
    return [['command:'], ['event_log:', 'signal_hub:']]
  }
  if (gateId.startsWith('search.')) {
    return [['search:', 'projection:']]
  }

  if (providerShape === 'whatsapp_business_cloud') {
    if (gateId.startsWith('business_cloud.host_vault_')) {
      return [['vault_binding:']]
    }
    if (gateId.startsWith('business_cloud.edge_proxy_')) {
      return [['edge_proxy:']]
    }
    if (gateId === 'business_cloud.hermes_api_not_public') {
      return [['edge_proxy:', 'runtime_api:']]
    }
    if (gateId === 'business_cloud.inbound_message_webhook') {
      return [['event_log:', 'signal_hub:'], ['raw_record:']]
    }
    if (gateId === 'business_cloud.receipt_webhook_reconciliation') {
      return [['command:'], ['event_log:', 'signal_hub:'], ['raw_record:']]
    }
    if (gateId.startsWith('business_cloud.send_')) {
      return [['command:'], ['event_log:', 'signal_hub:']]
    }
    if (gateId === 'business_cloud.rate_limit_retry_hint') {
      return [['command:']]
    }
    if (gateId === 'business_cloud.not_personal_whatsapp_substitute') {
      return [['doc:', 'runtime_api:']]
    }
  }

  return [['audit:', 'doc:', 'event_log:', 'runtime_api:']]
}

function templateEvidenceRefs(providerShape, gateId, status) {
  const prefixGroups = requiredEvidenceRefPrefixGroups(providerShape, gateId)
  if (status !== 'passed') {
    return prefixGroups.map(
      (group) => `${group[0]}replace-with-sanitized-${gateId.replaceAll('.', '-')}-reference`
    )
  }
  return prefixGroups.map((group, index) => {
    const prefix = group[0]
    const suffix = `${providerShape}:${gateId.replaceAll('.', '-')}:${index + 1}`
    return `${prefix}${suffix}`
  })
}

function absolutePath(relativePath) {
  return path.isAbsolute(relativePath) ? relativePath : path.join(repoRoot, relativePath)
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function isIsoDate(value) {
  if (typeof value !== 'string' || value.trim() === '') {
    return false
  }
  const parsed = Date.parse(value)
  return Number.isFinite(parsed)
}

function makeGateTemplate(providerShape, gateId, status) {
  const evidenceRefs = templateEvidenceRefs(providerShape, gateId, status)
  return {
    status,
    observed_at: '2026-06-26T00:00:00.000Z',
    evidence_ref: evidenceRefs[0],
    evidence_refs: evidenceRefs,
    notes: 'sanitized evidence only; no account ids, phone numbers, message bodies, tokens, cookies, session material, media keys or provider URLs',
  }
}

function templateEvidence(providerShape = 'whatsapp_native_md', status = 'pending') {
  const evidence = {}
  for (const gateId of requiredGateIds(providerShape)) {
    evidence[gateId] = makeGateTemplate(providerShape, gateId, status)
  }
  return {
    schema_version: 1,
    run_id: 'replace-with-local-run-id',
    generated_at: new Date('2026-06-26T00:00:00.000Z').toISOString(),
    provider_shape: providerShape,
    runtime_kind: 'replace-with-runtime-kind',
    account_fingerprint: 'sha256:0000000000000000000000000000000000000000000000000000000000000000',
    operator_attestation: {
      low_risk_or_test_account: status === 'passed',
      owner_visible_runtime: status === 'passed',
      no_hidden_or_headless_runtime: status === 'passed',
      secrets_not_recorded: status === 'passed',
      no_direct_domain_mutation: status === 'passed',
    },
    evidence,
  }
}

function evidenceRefs(gate) {
  const refs = []
  if (typeof gate.evidence_ref === 'string') {
    refs.push(gate.evidence_ref)
  }
  if (Array.isArray(gate.evidence_refs)) {
    for (const item of gate.evidence_refs) {
      if (typeof item === 'string') {
        refs.push(item)
      }
    }
  }
  return Array.from(new Set(refs.map((item) => item.trim()).filter(Boolean)))
}

function evidenceRefErrors(providerShape, gateId, gate) {
  const refs = evidenceRefs(gate)
  const errors = []
  if (refs.length === 0) {
    return [`evidence.${gateId}.evidence_refs must include sanitized references`]
  }
  for (const ref of refs) {
    if (/replace-with|pending|todo|example|dummy|placeholder/i.test(ref)) {
      errors.push(`evidence.${gateId}.evidence_ref contains placeholder value: ${ref}`)
    }
    if (!allowedEvidenceRefPrefixes.some((prefix) => ref.startsWith(prefix))) {
      errors.push(
        `evidence.${gateId}.evidence_ref must start with one of ${allowedEvidenceRefPrefixes.join(', ')}: ${ref}`
      )
    }
  }
  const prefixGroups = requiredEvidenceRefPrefixGroups(providerShape, gateId)
  for (const group of prefixGroups) {
    if (!refs.some((ref) => group.some((prefix) => ref.startsWith(prefix)))) {
      errors.push(
        `evidence.${gateId}.evidence_refs must include at least one ${group.join(' or ')} reference`
      )
    }
  }
  return errors
}

function collectValidationErrors(document) {
  const errors = []
  if (!isPlainObject(document)) {
    return ['evidence root must be a JSON object']
  }

  if (document.schema_version !== 1) {
    errors.push('schema_version must be 1')
  }
  if (typeof document.run_id !== 'string' || document.run_id.trim() === '') {
    errors.push('run_id must be a non-empty string')
  }
  if (!isIsoDate(document.generated_at)) {
    errors.push('generated_at must be an ISO-like timestamp')
  }
  if (!providerShapes.has(document.provider_shape)) {
    errors.push(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  if (typeof document.runtime_kind !== 'string' || document.runtime_kind.trim() === '') {
    errors.push('runtime_kind must be a non-empty string')
  }
  if (
    typeof document.account_fingerprint !== 'string'
    || !/^sha256:[a-f0-9]{64}$/i.test(document.account_fingerprint)
  ) {
    errors.push('account_fingerprint must be sha256:<64 hex chars>, not a raw account id or phone number')
  } else if (/^sha256:0{64}$/i.test(document.account_fingerprint)) {
    errors.push('account_fingerprint must be replaced with the real hashed account fingerprint')
  }

  const attestation = document.operator_attestation
  if (!isPlainObject(attestation)) {
    errors.push('operator_attestation must be an object')
  } else {
    for (const key of [
      'low_risk_or_test_account',
      'owner_visible_runtime',
      'no_hidden_or_headless_runtime',
      'secrets_not_recorded',
      'no_direct_domain_mutation',
    ]) {
      if (attestation[key] !== true) {
        errors.push(`operator_attestation.${key} must be true`)
      }
    }
  }

  if (!isPlainObject(document.evidence)) {
    errors.push('evidence must be an object keyed by gate id')
  } else if (providerShapes.has(document.provider_shape)) {
    for (const gateId of requiredGateIds(document.provider_shape)) {
      const gate = document.evidence[gateId]
      if (!isPlainObject(gate)) {
        errors.push(`evidence.${gateId} is required`)
        continue
      }
      if (gate.status !== 'passed') {
        errors.push(`evidence.${gateId}.status must be passed`)
      }
      if (!isIsoDate(gate.observed_at)) {
        errors.push(`evidence.${gateId}.observed_at must be an ISO-like timestamp`)
      }
      errors.push(...evidenceRefErrors(document.provider_shape, gateId, gate))
    }
  }

  const serialized = JSON.stringify(document)
  for (const pattern of secretLikePatterns) {
    if (pattern.test(serialized)) {
      errors.push(`evidence artifact contains forbidden secret/private marker: ${pattern.source}`)
    }
  }

  return errors
}

function printResult(ok, checks, extra = {}) {
  console.log(
    JSON.stringify(
      {
        ok,
        generated_at: new Date().toISOString(),
        evidence_path: evidencePath,
        ...extra,
        checks,
      },
      null,
      2
    )
  )
}

function runTemplate() {
  const providerShape =
    argValue('--provider-shape')
    ?? process.env.HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim()
    ?? 'whatsapp_native_md'
  const status = argValue('--status') ?? 'pending'
  if (!providerShapes.has(providerShape)) {
    console.error(`provider shape must be one of ${Array.from(providerShapes).join(', ')}`)
    process.exitCode = 1
    return
  }
  if (!['pending', 'passed'].includes(status)) {
    console.error('template status must be pending or passed')
    process.exitCode = 1
    return
  }
  console.log(JSON.stringify(templateEvidence(providerShape, status), null, 2))
}

function runSelfTest() {
  const valid = templateEvidence('whatsapp_native_md', 'passed')
  valid.run_id = 'self-test-native-md'
  valid.runtime_kind = 'native_md_smoke'
  valid.account_fingerprint =
    'sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'
  const validErrors = collectValidationErrors(valid)

  const leaked = structuredClone(valid)
  leaked.evidence['redaction.logs'].notes = 'contains access_token_value'
  const leakedErrors = collectValidationErrors(leaked)

  const weakRefs = structuredClone(valid)
  weakRefs.evidence['commands.no_completion_without_provider_observed_evidence'].evidence_refs = [
    'runtime_api:only-runtime-api-is-too-weak',
  ]
  weakRefs.evidence['commands.no_completion_without_provider_observed_evidence'].evidence_ref =
    'runtime_api:only-runtime-api-is-too-weak'
  const weakRefErrors = collectValidationErrors(weakRefs)

  const placeholderRefs = structuredClone(valid)
  placeholderRefs.evidence['event_flow.accepted_signal_events'].evidence_refs = [
    'event_log:replace-with-sanitized-event-reference',
  ]
  placeholderRefs.evidence['event_flow.accepted_signal_events'].evidence_ref =
    'event_log:replace-with-sanitized-event-reference'
  const placeholderRefErrors = collectValidationErrors(placeholderRefs)

  const checks = [
    {
      id: 'valid_synthetic_evidence_passes',
      status: validErrors.length === 0 ? 'pass' : 'fail',
      evidence: validErrors.length === 0 ? ['synthetic passed evidence validates'] : validErrors,
    },
    {
      id: 'secret_like_evidence_fails',
      status: leakedErrors.length > 0 ? 'pass' : 'fail',
      evidence:
        leakedErrors.length > 0
          ? ['synthetic access-token marker was rejected']
          : ['synthetic access-token marker was not rejected'],
    },
    {
      id: 'weak_reconciliation_refs_fail',
      status: weakRefErrors.length > 0 ? 'pass' : 'fail',
      evidence:
        weakRefErrors.length > 0
          ? ['synthetic command evidence without command/observed-event refs was rejected']
          : ['synthetic command evidence without command/observed-event refs was not rejected'],
    },
    {
      id: 'placeholder_refs_fail',
      status: placeholderRefErrors.length > 0 ? 'pass' : 'fail',
      evidence:
        placeholderRefErrors.length > 0
          ? ['synthetic placeholder evidence refs were rejected']
          : ['synthetic placeholder evidence refs were not rejected'],
    },
  ]
  const ok = checks.every((check) => check.status === 'pass')
  printResult(ok, checks, { mode: 'self-test' })
  if (!ok) {
    process.exitCode = 1
  }
}

function runValidation() {
  const filePath = absolutePath(evidencePath)
  if (!fs.existsSync(filePath)) {
    printResult(false, [
      {
        id: 'evidence_file_exists',
        status: 'fail',
        evidence: [
          `${evidencePath} does not exist; create a sanitized local evidence artifact after manual live smoke`,
        ],
      },
    ])
    process.exitCode = 1
    return
  }

  let document
  try {
    document = JSON.parse(fs.readFileSync(filePath, 'utf8'))
  } catch (error) {
    printResult(false, [
      {
        id: 'evidence_json_parse',
        status: 'fail',
        evidence: [error instanceof Error ? error.message : 'failed to parse evidence JSON'],
      },
    ])
    process.exitCode = 1
    return
  }

  const errors = collectValidationErrors(document)
  const ok = errors.length === 0
  printResult(ok, [
    {
      id: 'manual_live_smoke_evidence_contract',
      status: ok ? 'pass' : 'fail',
      evidence: ok
        ? [
            `${document.provider_shape} evidence covers ${requiredGateIds(document.provider_shape).length} required gates`,
          ]
        : errors,
    },
  ])
  if (!ok) {
    process.exitCode = 1
  }
}

const args = new Set(process.argv.slice(2))
function argValue(name) {
  const argv = process.argv.slice(2)
  const index = argv.indexOf(name)
  if (index >= 0 && typeof argv[index + 1] === 'string') {
    return argv[index + 1]
  }
  const prefix = `${name}=`
  const inline = argv.find((item) => item.startsWith(prefix))
  return inline ? inline.slice(prefix.length) : null
}

if (args.has('--template')) {
  runTemplate()
} else if (args.has('--self-test')) {
  runSelfTest()
} else {
  runValidation()
}
