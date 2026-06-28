#!/usr/bin/env node

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const waRsVersion = '0.2.0'
const cratesIoProbe = process.env.HERMES_WA_RS_CRATES_IO_PROBE === '1'
const checks = []

const requiredApis = [
  {
    id: 'send_message_api',
    relativePath: 'src/send.rs',
    pattern: /pub\s+async\s+fn\s+send_message\s*\(/,
    evidence: 'Client::send_message is available for send_text/reply/react/unreact payload submission',
  },
  {
    id: 'revoke_message_api',
    relativePath: 'src/send.rs',
    pattern: /pub\s+async\s+fn\s+revoke_message\s*\(/,
    evidence: 'Client::revoke_message is available for delete/revoke submission',
  },
  {
    id: 'edit_message_api',
    relativePath: 'src/client.rs',
    pattern: /pub\s+async\s+fn\s+edit_message\s*\(/,
    evidence: 'Client::edit_message is available for edit submission',
  },
  {
    id: 'mark_as_read_api',
    relativePath: 'src/receipt.rs',
    pattern: /pub\s+async\s+fn\s+mark_as_read\s*\(/,
    evidence: 'Client::mark_as_read is available for read receipts',
  },
  {
    id: 'leave_group_api',
    relativePath: 'src/features/groups.rs',
    pattern: /pub\s+async\s+fn\s+leave\s*\(/,
    evidence: 'Client::groups().leave is available for leave_group',
  },
  {
    id: 'upload_api',
    relativePath: 'src/upload.rs',
    pattern: /pub\s+async\s+fn\s+upload\s*\(/,
    evidence: 'Client::upload is available for media/voice-note upload',
  },
  {
    id: 'download_from_params_api',
    relativePath: 'src/download.rs',
    pattern: /pub\s+async\s+fn\s+download_from_params\s*\(/,
    evidence: 'Client::download_from_params is available for media download refs',
  },
]

const unsupportedExpectations = [
  {
    id: 'no_status_publish_api',
    commandKinds: ['publish_status'],
    matches: (name) =>
      name === 'send_status'
      || name === 'publish_status'
      || (name.includes('status') && (name.includes('publish') || name.includes('post'))),
  },
  {
    id: 'no_dialog_state_write_api',
    commandKinds: ['archive', 'unarchive', 'mute', 'unmute', 'pin', 'unpin', 'mark_unread'],
    matches: (name) =>
      [
        'archive',
        'unarchive',
        'mute',
        'unmute',
        'pin',
        'unpin',
        'mark_unread',
        'mark_as_unread',
      ].includes(name),
  },
  {
    id: 'no_join_by_invite_api',
    commandKinds: ['join_group'],
    matches: (name) =>
      name === 'join_group'
      || name === 'join_by_invite'
      || name === 'accept_invite'
      || name === 'accept_group_invite',
  },
]

function pass(id, evidence) {
  checks.push({ id, status: 'pass', evidence })
}

function fail(id, evidence) {
  checks.push({ id, status: 'fail', evidence })
}

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function cargoDependencyVersion(cargoToml, crateName) {
  const escaped = crateName.replaceAll('-', String.raw`\-`)
  const pattern = new RegExp(`${escaped}\\s*=\\s*\\{[^\\n]*version\\s*=\\s*"([^"]+)"`)
  return cargoToml.match(pattern)?.[1] ?? null
}

function cargoLockPackageVersion(cargoLock, crateName) {
  const escaped = crateName.replaceAll('-', String.raw`\-`)
  const pattern = new RegExp(`name\\s*=\\s*"${escaped}"\\nversion\\s*=\\s*"([^"]+)"`, 'm')
  return cargoLock.match(pattern)?.[1] ?? null
}

function cargoInfoVersion(crateName) {
  const result = spawnSync('cargo', ['info', crateName], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 30_000,
  })
  if (result.status !== 0) {
    return {
      ok: false,
      evidence: [
        `cargo info ${crateName} failed with status ${result.status}`,
        ...(result.stderr?.trim() ? [result.stderr.trim().slice(0, 500)] : []),
      ],
    }
  }
  const version = result.stdout.match(/^version:\s*(?<version>\S+)/m)?.groups?.version
  if (!version) {
    return {
      ok: false,
      evidence: [`cargo info ${crateName} did not expose a version line`],
    }
  }
  return {
    ok: true,
    version,
    evidence: [`cargo info ${crateName} reports version ${version}`],
  }
}

function waRsSourceRoot() {
  if (process.env.HERMES_WA_RS_SOURCE_DIR?.trim()) {
    return process.env.HERMES_WA_RS_SOURCE_DIR.trim()
  }

  const cargoHome = process.env.CARGO_HOME?.trim() || path.join(os.homedir(), '.cargo')
  const registrySrc = path.join(cargoHome, 'registry', 'src')
  if (!fs.existsSync(registrySrc)) {
    return null
  }

  for (const registryNamespace of fs.readdirSync(registrySrc)) {
    const candidate = path.join(registrySrc, registryNamespace, `wa-rs-${waRsVersion}`)
    if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
      return candidate
    }
  }
  return null
}

function waRsCoreSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-core-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
    return candidate
  }
  return null
}

function waRsAppStateSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-appstate-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'lib.rs'))) {
    return candidate
  }
  return null
}

function waRsProtoSourceRoot(root) {
  const registryNamespace = path.dirname(root)
  const candidate = path.join(registryNamespace, `wa-rs-proto-${waRsVersion}`)
  if (fs.existsSync(path.join(candidate, 'src', 'whatsapp.rs'))) {
    return candidate
  }
  return null
}

function listRustFiles(root) {
  const result = []
  const stack = [root]
  while (stack.length > 0) {
    const current = stack.pop()
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const fullPath = path.join(current, entry.name)
      if (entry.isDirectory()) {
        stack.push(fullPath)
      } else if (entry.isFile() && entry.name.endsWith('.rs')) {
        result.push(fullPath)
      }
    }
  }
  return result.sort()
}

function publicSurfaceFiles(root) {
  const publicRoots = [
    path.join(root, 'src', 'bot.rs'),
    path.join(root, 'src', 'client.rs'),
    path.join(root, 'src', 'send.rs'),
    path.join(root, 'src', 'receipt.rs'),
    path.join(root, 'src', 'upload.rs'),
    path.join(root, 'src', 'download.rs'),
    path.join(root, 'src', 'features'),
  ]
  const files = []
  for (const publicRoot of publicRoots) {
    if (!fs.existsSync(publicRoot)) {
      continue
    }
    const stat = fs.statSync(publicRoot)
    if (stat.isDirectory()) {
      files.push(...listRustFiles(publicRoot))
    } else {
      files.push(publicRoot)
    }
  }
  return files.sort()
}

function publicFunctionNames(root) {
  const names = []
  for (const filePath of publicSurfaceFiles(root)) {
    const text = fs.readFileSync(filePath, 'utf8')
    const regex = /pub\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/g
    for (const match of text.matchAll(regex)) {
      names.push({
        name: match[1],
        file: path.relative(root, filePath),
      })
    }
  }
  return names
}

function publicFunctionNamesInFiles(root, files) {
  const names = []
  for (const filePath of files) {
    const text = fs.readFileSync(filePath, 'utf8')
    const regex = /pub\s+(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/g
    for (const match of text.matchAll(regex)) {
      names.push({
        name: match[1],
        file: path.relative(root, filePath),
      })
    }
  }
  return names
}

function requireContains(relativePath, pattern, id, evidence) {
  const text = fs.readFileSync(relativePath, 'utf8')
  if (pattern.test(text)) {
    pass(id, [evidence])
  } else {
    fail(id, [`${relativePath} does not match ${pattern}`])
  }
}

function verifyLowLevelGapEvidence(root) {
  requireContains(
    path.join(root, 'src', 'request.rs'),
    /pub\s+async\s+fn\s+send_iq\s*\(/,
    'custom_iq_api_exists_but_is_low_level',
    'Client::send_iq exists for custom IQ stanzas, but unsupported app-state commands still need a safe encoder and smoke evidence'
  )

  const coreRoot = waRsCoreSourceRoot(root)
  const appStateRoot = waRsAppStateSourceRoot(root)
  if (!coreRoot) {
    fail('wa_rs_core_source_available', [
      `wa-rs-core ${waRsVersion} source not found next to ${root}`,
    ])
  } else {
    pass('wa_rs_core_source_available', [`using ${coreRoot}`])
    const groupsSource = fs.readFileSync(path.join(coreRoot, 'src', 'iq', 'groups.rs'), 'utf8')
    const joinInviteMarkers = [
      'JoinGroupIq',
      'AcceptInvite',
      'accept_invite',
      'join_by_invite',
      'GroupInviteJoin',
    ]
    const present = joinInviteMarkers.filter((marker) => groupsSource.includes(marker))
    if (present.length === 0) {
      pass('no_join_by_invite_iq_spec', [
        'wa-rs-core group IQ surface has invite-link fetch/reset, but no join-by-invite/accept-invite IQ spec',
      ])
    } else {
      fail('no_join_by_invite_iq_spec', present.map((marker) => `found ${marker}`))
    }
  }

  if (!appStateRoot) {
    fail('wa_rs_appstate_source_available', [
      `wa-rs-appstate ${waRsVersion} source not found next to ${root}`,
    ])
    return
  }

  pass('wa_rs_appstate_source_available', [`using ${appStateRoot}`])
  const appStateLib = fs.readFileSync(path.join(appStateRoot, 'src', 'lib.rs'), 'utf8')
  const appStateFiles = listRustFiles(path.join(appStateRoot, 'src'))
  const encoderFile = appStateFiles.find((filePath) => path.basename(filePath) === 'encode.rs')
  const publicFns = publicFunctionNamesInFiles(appStateRoot, appStateFiles)
  const outgoingEncoderHits = publicFns.filter((item) =>
    /^(encode|encrypt|build|create|send)_.*(patch|mutation|app_state|syncd)/u.test(item.name)
    || /^(patch|mutation|app_state|syncd)_.*(encode|encrypt|build|create|send)/u.test(item.name)
  )
  const exportedEncodeModule = /\bpub\s+mod\s+encode\b/u.test(appStateLib)
  if (!encoderFile && !exportedEncodeModule && outgoingEncoderHits.length === 0) {
    pass('no_public_outgoing_appstate_encoder', [
      'wa-rs-appstate exposes decode/hash/process helpers but no public outgoing patch/mutation encoder for archive/mute/pin/unread/status writes',
    ])
  } else {
    fail('no_public_outgoing_appstate_encoder', [
      ...(encoderFile ? [`found ${path.relative(appStateRoot, encoderFile)}`] : []),
      ...(exportedEncodeModule ? ['lib.rs exports pub mod encode'] : []),
      ...outgoingEncoderHits.map((hit) => `${hit.file}: pub fn ${hit.name}`),
    ])
  }
}

function verifyForwardTextReemitContract(root) {
  const protoRoot = waRsProtoSourceRoot(root)
  if (!protoRoot) {
    fail('wa_rs_proto_source_available_for_forward_context', [
      `wa-rs-proto ${waRsVersion} source not found next to ${root}`,
    ])
    return
  }

  pass('wa_rs_proto_source_available_for_forward_context', [`using ${protoRoot}`])
  requireContains(
    path.join(protoRoot, 'src', 'whatsapp.rs'),
    /pub forwarding_score: ::core::option::Option<u32>/,
    'wa_rs_proto_forwarding_score_available',
    'ExtendedTextMessage.ContextInfo exposes forwarding_score for forwarded text reemit'
  )
  requireContains(
    path.join(protoRoot, 'src', 'whatsapp.rs'),
    /pub is_forwarded: ::core::option::Option<bool>/,
    'wa_rs_proto_is_forwarded_available',
    'ExtendedTextMessage.ContextInfo exposes is_forwarded for forwarded text reemit'
  )
}

function verifyLocalWaRsApi(root) {
  pass('wa_rs_source_available', [`using ${root}`])

  for (const api of requiredApis) {
    requireContains(
      path.join(root, api.relativePath),
      api.pattern,
      api.id,
      `${api.relativePath}: ${api.evidence}`
    )
  }

  const publicFns = publicFunctionNames(root)
  for (const expectation of unsupportedExpectations) {
    const hits = publicFns.filter((item) => expectation.matches(item.name))
    if (hits.length === 0) {
      pass(expectation.id, [
        `${expectation.commandKinds.join(', ')} remains unsupported by wa-rs ${waRsVersion} public safe API inventory`,
      ])
    } else {
      fail(
        expectation.id,
        hits.map((hit) => `${hit.file}: pub fn ${hit.name} may support ${expectation.commandKinds.join(', ')}`)
      )
    }
  }

  verifyLowLevelGapEvidence(root)
  verifyForwardTextReemitContract(root)
}

function verifyRustAndCrateUpgradeContext() {
  const cargoToml = readText('backend/Cargo.toml')
  const cargoLock = readText('Cargo.lock')
  const status = readText('docs/integrations/whatsapp/status.md')
  const nativeMd = readText('backend/src/integrations/whatsapp/runtime/native_md.rs')

  const rustVersion = cargoToml.match(/^rust-version\s*=\s*"(?<version>[^"]+)"/m)?.groups
    ?.version
  if (
    rustVersion === '1.89'
    && status.includes('Rust 1.88 провален')
    && status.includes('MSRV поднят до Rust 1.89')
  ) {
    pass('native_md_rust_baseline_context', [
      'backend/Cargo.toml pins rust-version 1.89 after the Rust 1.88 compile spike failed',
    ])
  } else {
    fail('native_md_rust_baseline_context', [
      `expected backend rust-version 1.89 with status documentation; found ${rustVersion ?? '<missing>'}`,
    ])
  }

  const waRsCargoVersion = cargoDependencyVersion(cargoToml, 'wa-rs')
  const waRsCoreCargoVersion = cargoDependencyVersion(cargoToml, 'wa-rs-core')
  const waRsLockVersion = cargoLockPackageVersion(cargoLock, 'wa-rs')
  const waRsCoreLockVersion = cargoLockPackageVersion(cargoLock, 'wa-rs-core')
  if (
    waRsCargoVersion === waRsVersion
    && waRsCoreCargoVersion === waRsVersion
    && waRsLockVersion === waRsVersion
    && waRsCoreLockVersion === waRsVersion
  ) {
    pass('native_md_wa_rs_dependency_context', [
      `backend/Cargo.toml and Cargo.lock both pin wa-rs/wa-rs-core ${waRsVersion}`,
    ])
  } else {
    fail('native_md_wa_rs_dependency_context', [
      `wa-rs dependency context mismatch: Cargo.toml wa-rs=${waRsCargoVersion ?? '<missing>'}, wa-rs-core=${waRsCoreCargoVersion ?? '<missing>'}, Cargo.lock wa-rs=${waRsLockVersion ?? '<missing>'}, wa-rs-core=${waRsCoreLockVersion ?? '<missing>'}`,
    ])
  }

  if (
    nativeMd.includes('"public_availability": "blocked_until_manual_live_smoke_and_missing_safe_write_apis_are_resolved"')
    && nativeMd.includes('"wa_rs_surface": "no public status_publish API found"')
    && nativeMd.includes('ArchiveUpdate is inbound app-state dispatch only')
    && nativeMd.includes('groups API exposes create/add/remove/admin/link/leave but no join-by-invite API')
  ) {
    pass('native_md_upgrade_requires_provider_api_not_toolchain_only', [
      'native_md health keeps unsupported commands blocked on safe provider API plus smoke evidence, not on Rust toolchain version alone',
    ])
  } else {
    fail('native_md_upgrade_requires_provider_api_not_toolchain_only', [
      'native_md health no longer documents the safe-provider-API gate for the remaining unsupported commands',
    ])
  }
}

function verifyOptionalPublishedCrateProbe() {
  if (!cratesIoProbe) {
    pass('native_md_crates_io_probe', [
      'crates.io probe disabled; set HERMES_WA_RS_CRATES_IO_PROBE=1 to verify published wa-rs crate versions with cargo info',
    ])
    return
  }

  const crateNames = ['wa-rs', 'wa-rs-core', 'wa-rs-appstate']
  const evidence = []
  for (const crateName of crateNames) {
    const result = cargoInfoVersion(crateName)
    evidence.push(...result.evidence)
    if (!result.ok) {
      fail('native_md_crates_io_probe', evidence)
      return
    }
    if (result.version !== waRsVersion) {
      fail('native_md_crates_io_probe', [
        ...evidence,
        `${crateName} published version ${result.version} does not match expected ${waRsVersion}; inspect the new SDK before changing native_md support claims`,
      ])
      return
    }
  }
  pass('native_md_crates_io_probe', evidence)
}

function verifyHermesGapManifest() {
  const nativeMd = readText('backend/src/integrations/whatsapp/runtime/native_md.rs')
  const requiredMarkers = [
    'const NATIVE_MD_VERIFIED_PROVIDER_COMMANDS',
    'const NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS',
    'native_md_wa_rs_sdk_command_gap_health',
    '"evidence_basis": "local_crate_source_public_api_inventory"',
    '"unsupported_execution_policy"',
    '"completion_rule": "never_completed_without_provider_observed_event"',
    '"command_kind": "forward"',
    '"submission_mode": "forwarded_text_reemit"',
    'native_md_forward_text_message',
    'forwarding_score: Some(1)',
    'is_forwarded: Some(true)',
  ]
  const missing = requiredMarkers.filter((marker) => !nativeMd.includes(marker))
  if (missing.length === 0) {
    pass('hermes_native_md_gap_manifest', [
      'native_md health exposes verified/unsupported command matrix and unsupported execution policy',
    ])
  } else {
    fail('hermes_native_md_gap_manifest', missing.map((marker) => `missing ${marker}`))
  }

  const status = readText('docs/integrations/whatsapp/status.md')
  const adr0101 = readText('docs/adr/ADR-0101-whatsapp-provider-runtime-selection.md')
  const gapAnalysis = readText('docs/integrations/whatsapp/gap-analysis.md')
  const statusMarkers = [
    'native wa-rs command gap manifest',
    'remaining native SDK command gap verifier',
    'native low-level SDK gap evidence',
    'native forward text reemit submission',
    'Status/archive/mute/pin/join/unread remain',
  ]
  const missingStatus = statusMarkers.filter((marker) => !status.includes(marker))
  if (missingStatus.length === 0) {
    pass('docs_track_native_md_gap_verifier', [
      'docs/integrations/whatsapp/status.md tracks the native SDK command gap verifier and remaining unsupported writes',
    ])
  } else {
    fail('docs_track_native_md_gap_verifier', missingStatus.map((marker) => `missing ${marker}`))
  }

  const adrMarkers = [
    'Forward is supported only as smoke-gated forwarded-text',
    'reemit: Communications projection text',
    'ContextInfo.is_forwarded = true',
    'forwarding_score = 1',
    'forwarded-text reemit contract for `forward`',
    'Status/',
    'archive/mute/pin/join/unread remain structured unsupported paths',
  ]
  const missingAdr = adrMarkers.filter((marker) => !adr0101.includes(marker))
  const staleAdrMarkers = [
    'missing safe write APIs for forward',
    'Forward/status/',
    'Forward/status/archive/mute/pin/join/unread remain',
  ].filter((marker) => adr0101.includes(marker))
  if (missingAdr.length === 0 && staleAdrMarkers.length === 0) {
    pass('adr_0101_tracks_forward_text_reemit_gap', [
      'ADR-0101 treats forward as smoke-gated forwarded-text reemit and keeps only status/dialog/join/unread style gaps unsupported',
    ])
  } else {
    fail('adr_0101_tracks_forward_text_reemit_gap', [
      ...missingAdr.map((marker) => `missing ${marker}`),
      ...staleAdrMarkers.map((marker) => `stale ${marker}`),
    ])
  }

  const gapMarkers = [
    '| Forwards | PARTIAL |',
    'forwarded-text reemit',
    'status/archive/mute/pin/join/unread live commands remain open',
  ]
  const missingGapMarkers = gapMarkers.filter((marker) => !gapAnalysis.includes(marker))
  if (missingGapMarkers.length === 0 && !gapAnalysis.includes('| Forwards | MISSING |')) {
    pass('gap_analysis_tracks_forward_partial_support', [
      'docs/integrations/whatsapp/gap-analysis.md no longer lists forward as fully missing after native text reemit support',
    ])
  } else {
    fail('gap_analysis_tracks_forward_partial_support', [
      ...missingGapMarkers.map((marker) => `missing ${marker}`),
      ...(gapAnalysis.includes('| Forwards | MISSING |') ? ['stale | Forwards | MISSING |'] : []),
    ])
  }
}

const root = waRsSourceRoot()
verifyRustAndCrateUpgradeContext()
verifyOptionalPublishedCrateProbe()
if (!root) {
  fail('wa_rs_source_available', [
    `wa-rs ${waRsVersion} source not found under CARGO_HOME; run cargo fetch or set HERMES_WA_RS_SOURCE_DIR`,
  ])
} else {
  verifyLocalWaRsApi(root)
}
verifyHermesGapManifest()

const failed = checks.filter((check) => check.status === 'fail')
const result = {
  ok: failed.length === 0,
  generated_at: new Date().toISOString(),
  wa_rs_version: waRsVersion,
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (failed.length > 0) {
  process.exitCode = 1
}
