#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const args = new Set(process.argv.slice(2))
const requireClosed =
  args.has('--require-closed') || process.env.HERMES_WHATSAPP_REQUIRE_DOMAIN_CLOSED === '1'
const evidenceDir =
  process.env.HERMES_WHATSAPP_DOMAIN_CLOSURE_EVIDENCE_DIR ?? '.local/whatsapp'

const requiredEvidenceShapes = [
  'whatsapp_native_md',
  'whatsapp_web_companion',
  'whatsapp_business_cloud',
]

const checks = []
const blockers = []

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function check(id, status, evidence) {
  checks.push({ id, status, evidence })
}

function pass(id, evidence) {
  check(id, 'pass', evidence)
}

function fail(id, evidence) {
  check(id, 'fail', evidence)
}

function block(id, evidence) {
  blockers.push({ id, evidence })
  check(id, 'blocked', evidence)
}

function requireContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const missing = needles.filter((needle) => !text.includes(needle))
  if (missing.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} contains ${needle}`))
  } else {
    fail(id, missing.map((needle) => `${relativePath} missing ${needle}`))
  }
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function evidenceFiles() {
  const absoluteEvidenceDir = path.isAbsolute(evidenceDir)
    ? evidenceDir
    : path.join(repoRoot, evidenceDir)
  if (!fs.existsSync(absoluteEvidenceDir)) {
    return []
  }
  return fs
    .readdirSync(absoluteEvidenceDir)
    .filter((name) => /^live-smoke-evidence.*\.json$/u.test(name))
    .map((name) => path.join(absoluteEvidenceDir, name))
    .sort()
}

function validateEvidenceFile(filePath) {
  const run = spawnSync(process.execPath, ['scripts/whatsapp-live-smoke-evidence.mjs'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      HERMES_WHATSAPP_LIVE_SMOKE_EVIDENCE: filePath,
    },
    encoding: 'utf8',
  })

  let document = null
  try {
    document = JSON.parse(fs.readFileSync(filePath, 'utf8'))
  } catch {
    return {
      ok: false,
      providerShape: '<unparseable>',
      filePath,
      evidence: [`${filePath} is not parseable JSON`],
    }
  }

  const providerShape = isPlainObject(document) ? document.provider_shape : '<missing>'
  if (run.status === 0) {
    return {
      ok: true,
      providerShape,
      filePath,
      evidence: [`${filePath} passed scripts/whatsapp-live-smoke-evidence.mjs`],
    }
  }

  const stdout = run.stdout?.trim()
  const stderr = run.stderr?.trim()
  return {
    ok: false,
    providerShape,
    filePath,
    evidence: [
      `${filePath} failed scripts/whatsapp-live-smoke-evidence.mjs`,
      ...(stdout ? [`stdout: ${stdout.slice(0, 800)}`] : []),
      ...(stderr ? [`stderr: ${stderr.slice(0, 800)}`] : []),
    ],
  }
}

function nativeUnsupportedCommands() {
  const nativeMd = readText('backend/src/integrations/whatsapp/runtime/native_md.rs')
  const match = nativeMd.match(
    /const\s+NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS:\s*&\[\s*&str\s*\]\s*=\s*&\[(?<body>[\s\S]*?)\];/u
  )
  if (!match?.groups?.body) {
    return null
  }
  return Array.from(match.groups.body.matchAll(/"([^"]+)"/gu), (item) => item[1]).sort()
}

function statusClosureState() {
  const status = readText('docs/whatsapp/status.md')
  if (status.includes('DOMAIN CLOSURE          = achieved')) {
    return 'achieved'
  }
  if (status.includes('DOMAIN CLOSURE          = not achieved')) {
    return 'not_achieved'
  }
  return 'unknown'
}

function adr0101State() {
  const adr = readText('docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md')
  const match = adr.match(/^Status:\s*(?<status>.+)$/mu)
  return match?.groups?.status?.trim() ?? 'unknown'
}

requireContains('static_readiness_targets_exist', 'Makefile', [
  'whatsapp-live-smoke-readiness:',
  'whatsapp-native-md-sdk-gap-readiness:',
  'whatsapp-live-smoke-evidence:',
  'whatsapp-business-cloud-edge-readiness:',
])

requireContains('acceptance_docs_track_current_blockers', 'docs/whatsapp/status.md', [
  'DOMAIN CLOSURE          = not achieved',
  'manual smoke',
  'remaining safe write APIs',
  'WebView live smoke',
  'Business Cloud public exposure/smoke',
])

requireContains('architecture_guard_contract_exists', 'backend/tests/communications_architecture_target.rs', [
  'whatsapp_provider_runtime_is_replaceable_trait_boundary',
  'runtime/native_md',
  'runtime/business_cloud',
  'domains',
  'engines',
])

requireContains('signal_hub_fixture_contract_exists', 'backend/tests/whatsapp_signal_hub.rs', [
  'sanitized WhatsApp event payload must remove',
  'signal.accepted.whatsapp',
  'provider-observed reconciliation',
  'whatsapp_native_md_unsupported_write_gap_is_explicit_and_structured',
  'unsupported writes',
])

requireContains('manual_smoke_evidence_contract_exists', 'scripts/whatsapp-live-smoke-evidence.mjs', [
  'commonGateIds',
  'personalGateIds',
  'businessCloudGateIds',
  'allowedEvidenceRefPrefixes',
  'requiredEvidenceRefPrefixGroups',
  'evidence_refs',
  'account_fingerprint must be sha256:<64 hex chars>',
  'evidence.${gateId}.status must be passed',
  'weak_reconciliation_refs_fail',
  'placeholder_refs_fail',
])

requireContains('native_md_upgrade_path_context_exists', 'scripts/whatsapp-native-md-sdk-gap-readiness.mjs', [
  'verifyRustAndCrateUpgradeContext()',
  'native_md_rust_baseline_context',
  'native_md_wa_rs_dependency_context',
  'native_md_crates_io_probe',
  'native_md_upgrade_requires_provider_api_not_toolchain_only',
])

requireContains('native_md_upgrade_docs_track_toolchain_limit', 'docs/whatsapp/status.md', [
  'native Rust/wa-rs upgrade path verifier',
  'Rust/toolchain upgrade is not treated as sufficient evidence',
  'HERMES_WA_RS_CRATES_IO_PROBE=1',
])

requireContains('live_smoke_evidence_collector_exists', 'scripts/whatsapp-live-smoke-collect-evidence.mjs', [
  'defaultObservationsPath = \'.local/whatsapp/live-smoke-observations.json\'',
  'whatsapp-live-smoke-evidence.mjs',
  '--observations-template',
  'assertNoSecretLikeContent',
  'mergeEvidence(template, observations)',
])

requireContains('live_smoke_evidence_collector_target_exists', 'Makefile', [
  'whatsapp-live-smoke-collect-evidence:',
  'node scripts/whatsapp-live-smoke-collect-evidence.mjs',
])

requireContains('live_smoke_evidence_collector_docs_exist', 'docs/whatsapp/live-smoke-checklist.md', [
  'make whatsapp-live-smoke-collect-evidence',
  'normalizer, not a bypass',
  'Gates without operator-provided sanitized',
])

requireContains('adr_0101_acceptance_scope_keeps_live_blocked', 'docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md', [
  'Status: Accepted',
  'Acceptance scope',
  'does not make any WhatsApp live',
  'remain blocked until their live-smoke evidence',
])

const validatedEvidence = new Map()
const invalidEvidence = []
for (const filePath of evidenceFiles()) {
  const result = validateEvidenceFile(filePath)
  if (result.ok && requiredEvidenceShapes.includes(result.providerShape)) {
    validatedEvidence.set(result.providerShape, result)
  } else {
    invalidEvidence.push(result)
  }
}

for (const providerShape of requiredEvidenceShapes) {
  const result = validatedEvidence.get(providerShape)
  if (result) {
    pass(`live_smoke_evidence.${providerShape}`, result.evidence)
  } else {
    const candidates = invalidEvidence
      .filter((item) => item.providerShape === providerShape)
      .flatMap((item) => item.evidence)
    block(`live_smoke_evidence.${providerShape}`, [
      candidates.length > 0
        ? candidates.join('\n')
        : `${evidenceDir}/live-smoke-evidence*.json has no valid ${providerShape} evidence artifact`,
    ])
  }
}

const unsupportedCommands = nativeUnsupportedCommands()
if (unsupportedCommands === null) {
  fail('native_md_unsupported_command_manifest', [
    'NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS manifest was not found',
  ])
} else if (unsupportedCommands.length === 0) {
  pass('native_md_unsupported_command_manifest', [
    'native_md unsupported command manifest is empty',
  ])
} else {
  block('native_md_unsupported_commands_remaining', [
    `native_md still marks unsupported commands: ${unsupportedCommands.join(', ')}`,
  ])
}

const adrState = adr0101State()
if (adrState === 'Accepted') {
  pass('adr_0101_accepted', ['ADR-0101 is Accepted'])
} else {
  block('adr_0101_not_accepted', [`ADR-0101 status is ${adrState}`])
}

const closureState = statusClosureState()
const noFailedChecks = checks.every((item) => item.status !== 'fail')
const closureEvidenceComplete = blockers.length === 0
const closureAchieved =
  noFailedChecks && closureEvidenceComplete && closureState === 'achieved' && adrState === 'Accepted'

if (closureAchieved) {
  pass('docs_status_claims_closure_only_after_evidence', [
    'docs/whatsapp/status.md claims DOMAIN CLOSURE = achieved and closure evidence is complete',
  ])
} else if (closureState === 'not_achieved') {
  pass('docs_status_does_not_overclaim_closure', [
    'docs/whatsapp/status.md keeps DOMAIN CLOSURE = not achieved while blockers remain',
  ])
} else if (closureState === 'achieved') {
  fail('docs_status_overclaims_closure', [
    'docs/whatsapp/status.md claims achieved but closure audit still has blockers',
  ])
} else {
  fail('docs_status_closure_state_unknown', [
    'docs/whatsapp/status.md must state DOMAIN CLOSURE = achieved or not achieved',
  ])
}

const result = {
  ok: noFailedChecks && (!requireClosed || closureAchieved),
  require_closed: requireClosed,
  closure_achieved: closureAchieved,
  generated_at: new Date().toISOString(),
  evidence_dir: evidenceDir,
  required_evidence_shapes: requiredEvidenceShapes,
  blockers,
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (!result.ok) {
  process.exitCode = 1
}
