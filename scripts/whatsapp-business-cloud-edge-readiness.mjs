#!/usr/bin/env node

import fs from 'node:fs'
import http from 'node:http'
import https from 'node:https'
import path from 'node:path'
import process from 'node:process'

const repoRoot = process.cwd()
const probeEdge = process.env.HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE === '1'
const probeReadyz = process.env.HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE === '1'
const edgeBaseUrl =
  process.env.HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_URL ?? 'http://127.0.0.1:8787'

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
    fail(id, missing.map((needle) => `${relativePath} missing ${needle}`))
  }
}

function requireNotContains(id, relativePath, needles) {
  const text = readText(relativePath)
  const present = needles.filter((needle) => text.includes(needle))
  if (present.length === 0) {
    pass(id, needles.map((needle) => `${relativePath} does not contain ${needle}`))
  } else {
    fail(id, present.map((needle) => `${relativePath} still contains ${needle}`))
  }
}

function parseJson(value) {
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

function request(method, pathname, headers = {}, body = '') {
  return new Promise((resolve, reject) => {
    const url = new URL(pathname, edgeBaseUrl)
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

async function probeLocalEdgeProxy() {
  if (!probeEdge) {
    pass('local_edge_proxy_probe', [
      'local edge probe disabled; set HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PROBE=1 to probe a running proxy',
    ])
    return
  }

  const health = await request('GET', '/healthz')
  const healthBody = parseJson(health.body)
  if (
    health.statusCode === 200
    && healthBody?.status === 'ok'
    && healthBody?.service === 'hermes-whatsapp-business-cloud-edge-proxy'
  ) {
    pass('local_edge_proxy_healthz', [`${edgeBaseUrl}/healthz returned ok`])
  } else {
    fail('local_edge_proxy_healthz', [
      `${edgeBaseUrl}/healthz returned ${health.statusCode}: ${health.body}`,
    ])
  }

  const manifest = await request('GET', '/manifest')
  const manifestBody = parseJson(manifest.body)
  if (
    manifest.statusCode === 200
    && manifestBody?.public_webhook_path === '/webhooks/whatsapp/business-cloud'
    && manifestBody?.protected_hermes_webhook_path
      === '/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks'
    && manifestBody?.protected_hermes_manifest_path
      === '/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest'
    && manifestBody?.local_auth_header === 'X-Hermes-Secret'
    && manifestBody?.signature_header === 'X-Hub-Signature-256'
    && manifestBody?.payload_policy === 'post_body_is_not_parsed_or_rewritten_by_edge_proxy'
  ) {
    pass('local_edge_proxy_manifest', [`${edgeBaseUrl}/manifest returned the protected forwarding contract`])
  } else {
    fail('local_edge_proxy_manifest', [
      `${edgeBaseUrl}/manifest returned ${manifest.statusCode}: ${manifest.body}`,
    ])
  }

  const unsignedPost = await request(
    'POST',
    '/webhooks/whatsapp/business-cloud',
    { 'Content-Type': 'application/json' },
    '{"entry":[]}'
  )
  const unsignedPostBody = parseJson(unsignedPost.body)
  if (
    unsignedPost.statusCode === 400
    && unsignedPostBody?.error === 'missing_x_hub_signature_256'
  ) {
    pass('local_edge_proxy_rejects_unsigned_post', [
      'unsigned Business Cloud POST is rejected before Hermes forwarding',
    ])
  } else {
    fail('local_edge_proxy_rejects_unsigned_post', [
      `unsigned POST returned ${unsignedPost.statusCode}: ${unsignedPost.body}`,
    ])
  }

  if (!probeReadyz) {
    pass('local_edge_proxy_readyz_probe', [
      'readyz probe disabled; set HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_READYZ_PROBE=1 when Hermes is running',
    ])
    return
  }

  const readyz = await request('GET', '/readyz')
  if (readyz.statusCode >= 200 && readyz.statusCode < 300) {
    pass('local_edge_proxy_readyz', [`${edgeBaseUrl}/readyz reached protected Hermes manifest`])
  } else {
    fail('local_edge_proxy_readyz', [
      `${edgeBaseUrl}/readyz returned ${readyz.statusCode}: ${readyz.body}`,
    ])
  }
}

requireContains('edge_proxy_public_surface', 'backend/src/bin/hermes_whatsapp_business_cloud_edge_proxy.rs', [
  'PUBLIC_WEBHOOK_PATH: &str = "/webhooks/whatsapp/business-cloud"',
  'PROTECTED_HERMES_WEBHOOK_PATH',
  'PROTECTED_HERMES_MANIFEST_PATH',
  'HERMES_SECRET_HEADER: &str = "X-Hermes-Secret"',
  'BUSINESS_CLOUD_SIGNATURE_HEADER: &str = "X-Hub-Signature-256"',
  '.route("/healthz", get(healthz))',
  '.route("/readyz", get(readyz))',
  '.route("/manifest", get(edge_manifest))',
  'PUBLIC_WEBHOOK_PATH,',
])

requireContains('edge_proxy_forwarding_contract', 'backend/src/bin/hermes_whatsapp_business_cloud_edge_proxy.rs', [
  'forward_hub_query_params_and_optional_account_id_to_protected_hermes',
  'forward_exact_raw_body_and_x_hub_signature_256_to_protected_hermes',
  'post_body_is_not_parsed_or_rewritten_by_edge_proxy',
  'missing_x_hub_signature_256',
  'hermes_url(PROTECTED_HERMES_WEBHOOK_PATH, None, false)',
  '.header(HERMES_SECRET_HEADER',
  '.header(BUSINESS_CLOUD_SIGNATURE_HEADER',
])

requireContains('edge_proxy_env_boundary', 'backend/src/bin/hermes_whatsapp_business_cloud_edge_proxy.rs', [
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET',
  'HERMES_LOCAL_API_SECRET',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID',
])

requireContains('edge_proxy_behavioral_tests_cover_contract', 'backend/src/bin/hermes_whatsapp_business_cloud_edge_proxy.rs', [
  'readyz_checks_manifest_without_account_scoping',
  'get_webhook_forwards_challenge_query_account_scope_and_local_secret',
  'post_webhook_forwards_raw_body_signature_and_no_account_query',
  'post_webhook_requires_meta_signature_before_forwarding',
  'missing_x_hub_signature_256',
])

requireContains('edge_proxy_signal_hub_static_guard', 'backend/tests/whatsapp_signal_hub.rs', [
  'whatsapp_business_cloud_proxy_manifest_keeps_hermes_protected',
  '/webhooks/whatsapp/business-cloud',
  'readyz_checks_manifest_without_account_scoping',
  'post_webhook_forwards_raw_body_signature_and_no_account_query',
])

requireContains('edge_proxy_compose_profile', 'docker/docker-compose.yml', [
  'whatsapp-business-cloud-edge-proxy:',
  'profiles:',
  'whatsapp-business-cloud-edge',
  'target: whatsapp-business-cloud-edge-proxy',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL',
  'HERMES_LOCAL_API_SECRET',
  '127.0.0.1}:${HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT:-8787}:8787',
  'curl -fsS http://127.0.0.1:8787/healthz',
])

requireContains('edge_proxy_dockerfile_target', 'docker/Dockerfile', [
  '--bin hermes-whatsapp-business-cloud-edge-proxy',
  'FROM debian:bookworm-slim AS whatsapp-business-cloud-edge-proxy',
  '/usr/local/bin/hermes-whatsapp-business-cloud-edge-proxy',
  'EXPOSE 8787',
])

requireContains('edge_proxy_env_example_is_loopback_and_non_secret', 'docker/.env.example', [
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND=127.0.0.1',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT=8787',
  'HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL=http://host.docker.internal:8080',
  '# HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID=optional-account-scope',
])

requireNotContains('edge_proxy_env_example_has_no_business_cloud_secret_values', 'docker/.env.example', [
  'whatsapp_business_cloud_access_token=',
  'whatsapp_business_cloud_app_secret=',
  'whatsapp_business_cloud_webhook_verify_token=',
])

requireContains('edge_proxy_makefile_targets', 'Makefile', [
  'whatsapp-business-cloud-edge-config:',
  'whatsapp-business-cloud-edge-up:',
  'whatsapp-business-cloud-edge-stop:',
  'whatsapp-business-cloud-edge-logs:',
])

requireContains('edge_proxy_docs_smoke_contract', 'docs/whatsapp/live-smoke-checklist.md', [
  'Business Cloud edge proxy checks',
  'make whatsapp-business-cloud-edge-config',
  'make whatsapp-business-cloud-edge-up',
  'GET /readyz',
  'Expose only the proxy path `/webhooks/whatsapp/business-cloud`',
  'do not expose Hermes `/api/v1` directly',
  'X-Hub-Signature-256',
])

requireContains('edge_proxy_status_tracks_remaining_public_gate', 'docs/whatsapp/status.md', [
  'DOMAIN CLOSURE          = not achieved',
  'Business Cloud public exposure/smoke',
  'business cloud edge proxy binary',
  'business cloud edge proxy behavioral contract',
  'business cloud edge proxy compose profile',
])

await probeLocalEdgeProxy().catch((error) => {
  fail('local_edge_proxy_probe_error', [error instanceof Error ? error.message : String(error)])
})

const failed = checks.filter((check) => check.status === 'fail')
const result = {
  ok: failed.length === 0,
  edge_probe: probeEdge,
  readyz_probe: probeReadyz,
  edge_base_url: edgeBaseUrl,
  generated_at: new Date().toISOString(),
  checks,
}

console.log(JSON.stringify(result, null, 2))

if (failed.length > 0) {
  process.exitCode = 1
}
