#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const defaultObservationsPath = '.local/whatsapp/live-smoke-observations.json'
const observationsPath =
  process.env.HERMES_WHATSAPP_LIVE_SMOKE_OBSERVATIONS ?? defaultObservationsPath
const providerShapes = new Set([
  'whatsapp_web_companion',
  'whatsapp_native_md',
  'whatsapp_business_cloud',
])
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

function absolutePath(relativePath) {
  return path.isAbsolute(relativePath) ? relativePath : path.join(repoRoot, relativePath)
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function sha256Fingerprint(value) {
  return `sha256:${createHash('sha256').update(value).digest('hex')}`
}

function providerShapeFrom(document) {
  const value =
    argValue('--provider-shape')
    ?? process.env.HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim()
    ?? document?.provider_shape
  if (!providerShapes.has(value)) {
    throw new Error(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  return value
}

function templateEvidence(providerShape) {
  const result = spawnSync(
    process.execPath,
    [
      'scripts/whatsapp-live-smoke-evidence.mjs',
      '--template',
      '--provider-shape',
      providerShape,
      '--status',
      'pending',
    ],
    {
      cwd: repoRoot,
      encoding: 'utf8',
    }
  )
  if (result.status !== 0) {
    throw new Error(
      `failed to render evidence template: ${(result.stderr || result.stdout).trim()}`
    )
  }
  return JSON.parse(result.stdout)
}

function observationsTemplate(providerShape) {
  const now = new Date('2026-06-26T00:00:00.000Z').toISOString()
  return {
    schema_version: 1,
    run_id: 'replace-with-local-run-id',
    generated_at: now,
    provider_shape: providerShape,
    runtime_kind: 'replace-with-runtime-kind',
    account_fingerprint: 'sha256:replace-with-64-hex-account-fingerprint',
    operator_attestation: {
      low_risk_or_test_account: false,
      owner_visible_runtime: false,
      no_hidden_or_headless_runtime: false,
      secrets_not_recorded: false,
      no_direct_domain_mutation: false,
    },
    evidence: {
      'preflight.readiness_target': {
        observed_at: now,
        evidence_refs: ['command:make-whatsapp-live-smoke-readiness-run-id'],
        notes: 'replace with sanitized local command/run reference',
      },
    },
  }
}

function outputPath(providerShape) {
  const explicit = process.env.HERMES_WHATSAPP_LIVE_SMOKE_EVIDENCE ?? argValue('--output')
  if (explicit?.trim()) {
    return absolutePath(explicit.trim())
  }
  return absolutePath(`.local/whatsapp/live-smoke-evidence-${providerShape}.json`)
}

function accountFingerprint(observations, providerShape) {
  if (
    typeof observations.account_fingerprint === 'string'
    && /^sha256:[a-f0-9]{64}$/i.test(observations.account_fingerprint)
  ) {
    return observations.account_fingerprint
  }
  const accountId = process.env.HERMES_WHATSAPP_SMOKE_ACCOUNT_ID?.trim()
  if (accountId) {
    return sha256Fingerprint(`${providerShape}:${accountId}`)
  }
  return observations.account_fingerprint ?? ''
}

function assertNoSecretLikeContent(document) {
  const serialized = JSON.stringify(document)
  for (const pattern of secretLikePatterns) {
    if (pattern.test(serialized)) {
      throw new Error(`observations contain forbidden secret/private marker: ${pattern.source}`)
    }
  }
}

function mergeEvidence(template, observations) {
  const observedEvidence = observations.evidence
  if (!isPlainObject(observedEvidence)) {
    throw new Error('observations.evidence must be an object keyed by gate id')
  }

  // Gates without operator-provided sanitized refs remain pending in the template.
  for (const [gateId, observedGate] of Object.entries(observedEvidence)) {
    if (!isPlainObject(template.evidence[gateId])) {
      throw new Error(`observations.evidence.${gateId} is not a known gate for ${template.provider_shape}`)
    }
    if (!isPlainObject(observedGate)) {
      throw new Error(`observations.evidence.${gateId} must be an object`)
    }
    const refs = Array.isArray(observedGate.evidence_refs)
      ? observedGate.evidence_refs.filter((item) => typeof item === 'string' && item.trim())
      : typeof observedGate.evidence_ref === 'string' && observedGate.evidence_ref.trim()
        ? [observedGate.evidence_ref.trim()]
        : []
    if (refs.length === 0) {
      throw new Error(`observations.evidence.${gateId}.evidence_refs must include sanitized refs`)
    }
    template.evidence[gateId] = {
      status: observedGate.status === 'pending' ? 'pending' : 'passed',
      observed_at: observedGate.observed_at ?? new Date().toISOString(),
      evidence_ref: refs[0],
      evidence_refs: Array.from(new Set(refs.map((item) => item.trim()))),
      notes:
        typeof observedGate.notes === 'string' && observedGate.notes.trim()
          ? observedGate.notes.trim()
          : 'sanitized live-smoke observation reference',
    }
  }
}

function validateEvidence(filePath) {
  const result = spawnSync(process.execPath, ['scripts/whatsapp-live-smoke-evidence.mjs'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      HERMES_WHATSAPP_LIVE_SMOKE_EVIDENCE: filePath,
    },
    encoding: 'utf8',
  })
  return {
    ok: result.status === 0,
    status: result.status,
    stdout: result.stdout ? JSON.parse(result.stdout) : null,
    stderr: result.stderr?.trim() ?? '',
  }
}

function writeObservationsTemplate() {
  const providerShape =
    argValue('--provider-shape')
    ?? process.env.HERMES_WHATSAPP_SMOKE_PROVIDER_SHAPE?.trim()
    ?? 'whatsapp_native_md'
  if (!providerShapes.has(providerShape)) {
    throw new Error(`provider_shape must be one of ${Array.from(providerShapes).join(', ')}`)
  }
  console.log(JSON.stringify(observationsTemplate(providerShape), null, 2))
}

function collectEvidence() {
  const observationsFile = absolutePath(observationsPath)
  if (!fs.existsSync(observationsFile)) {
    throw new Error(
      `${observationsPath} does not exist; create sanitized observations or run with --observations-template`
    )
  }

  const observations = readJson(observationsFile)
  assertNoSecretLikeContent(observations)
  const providerShape = providerShapeFrom(observations)
  const template = templateEvidence(providerShape)
  template.run_id = observations.run_id
  template.generated_at = new Date().toISOString()
  template.provider_shape = providerShape
  template.runtime_kind = observations.runtime_kind
  template.account_fingerprint = accountFingerprint(observations, providerShape)
  template.operator_attestation = observations.operator_attestation
  mergeEvidence(template, observations)
  assertNoSecretLikeContent(template)

  const filePath = outputPath(providerShape)
  fs.mkdirSync(path.dirname(filePath), { recursive: true })
  fs.writeFileSync(filePath, `${JSON.stringify(template, null, 2)}\n`)
  const validation = validateEvidence(filePath)
  console.log(
    JSON.stringify(
      {
        ok: validation.ok,
        generated_at: new Date().toISOString(),
        observations_path: observationsPath,
        evidence_path: path.relative(repoRoot, filePath),
        provider_shape: providerShape,
        validation: validation.stdout ?? validation.stderr,
      },
      null,
      2
    )
  )
  if (!validation.ok) {
    process.exitCode = 1
  }
}

try {
  if (process.argv.includes('--observations-template')) {
    writeObservationsTemplate()
  } else {
    collectEvidence()
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error))
  process.exitCode = 1
}
