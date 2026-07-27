import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const sources = {
  adr: new URL(
    'docs/adr/ADR-0300-loopback-full-stack-development-assembly.md',
    PROJECT_ROOT,
  ),
  rootMakefile: new URL('Makefile', PROJECT_ROOT),
  backendMakefile: new URL('Makefile', BACKEND_ROOT),
  assembly: new URL('scripts/dev-ensemble.sh', BACKEND_ROOT),
  release: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  probe: new URL('scripts/probe-dev-gateway.mjs', BACKEND_ROOT),
  vite: new URL('frontend/vite.config.ts', PROJECT_ROOT),
  gateway: new URL('src/kernel/src/platform/gateway.rs', BACKEND_ROOT),
  cli: new URL('src/kernel/src/cli/mod.rs', BACKEND_ROOT),
  protocol: new URL(
    'src/api/gateway/contracts/proto/hermes/gateway/v1/browser_session.proto',
    BACKEND_ROOT,
  ),
};

test('root make dev owns one loopback full-stack browser assembly', async () => {
  const {
    adr,
    rootMakefile,
    backendMakefile,
    assembly,
    release,
    probe,
    vite,
    gateway,
    cli,
    protocol,
  } = Object.fromEntries(
    await Promise.all(
      Object.entries(sources).map(async ([name, path]) => [
        name,
        await readFile(path, 'utf8'),
      ]),
    ),
  );

  assert.match(adr, /loopback_full_stack_dev_assembly_v1/);
  assert.match(rootMakefile, /build test dev docker tauri clean:[\s\S]*\$\(MAKE\) -C backend \$@/);
  assert.match(backendMakefile, /^dev:\n\t@\.\/scripts\/dev-ensemble\.sh$/m);
  assert.doesNotMatch(backendMakefile, /wait -n/);

  assert.match(assembly, /materialize-dev-release\.sh/);
  assert.match(assembly, /development\/authenticated\/compose\.yaml/);
  assert.match(assembly, /run_compose up --detach --wait/);
  assert.match(assembly, /provision-platform/);
  assert.match(assembly, /start-ensemble/);
  assert.match(assembly, /Admitting the exact Communications and provider module plan/);
  assert.match(assembly, /--browser-gateway-listen-address "\$gateway_address"/);
  assert.match(assembly, /--browser-gateway-development-proxy-proof-file "\$proof_file"/);
  assert.match(assembly, /HERMES_DEV_GATEWAY_PROOF_FILE="\$proof_file"/);
  assert.match(assembly, /probe-dev-gateway\.mjs/);
  assert.match(assembly, /curl .*"\$browser_origin\/readyz"/);
  assert.ok(
    assembly.indexOf('open "$browser_url"')
      > assembly.indexOf('Hermes development ensemble is ready'),
  );
  assert.match(assembly, /trap cleanup EXIT/);
  assert.doesNotMatch(assembly, /wait -n|0\.0\.0\.0|--browser-gateway-development-proxy-proof [^"-]/);

  assert.match(release, /hermes-communications-assembly/);
  assert.match(release, /hermes-attachment-security-assembly/);
  assert.match(release, /hermes-mail-assembly/);
  assert.match(release, /hermes-telegram-assembly/);
  assert.match(release, /hermes-whatsapp-assembly/);
  assert.match(release, /hermes-zulip-assembly/);
  assert.match(release, /build-distribution-release\.mjs/);

  assert.match(probe, /host: '127\.0\.0\.1'/);
  assert.match(probe, /origin: 'http:\/\/127\.0\.0\.1:5173'/);
  assert.match(probe, /'x-hermes-development-proxy-proof': proof/);
  assert.doesNotMatch(probe, /console\.(?:log|error)|process\.stdout|process\.stderr/);

  assert.match(vite, /host: '127\.0\.0\.1'/);
  assert.match(vite, /strictPort: true/);
  assert.match(vite, /'\^\/hermes\\\\\.'/);
  assert.match(vite, /'\/api\/realtime\/v1\/events'/);
  assert.match(vite, /'\/healthz'/);
  assert.match(vite, /'\/readyz'/);
  assert.match(vite, /request\.setHeader\(DEVELOPMENT_PROXY_PROOF_HEADER, gateway\.proof\)/);
  assert.doesNotMatch(vite, /define:\s*\{|VITE_.*PROOF/);

  assert.match(gateway, /BrowserGatewayExposureV1::LoopbackDevelopmentProxy/);
  assert.match(gateway, /starts_signed_development_foundation/);
  assert.match(gateway, /GatewayLoopbackListenerV1::bind/);
  assert.match(gateway, /with_loopback_development_proxy_policy/);
  assert.match(cli, /browser_gateway_development_proxy_proof_file: Option<PathBuf>/);
  assert.match(cli, /metadata\.permissions\(\)\.mode\(\) & 0o077/);
  assert.match(protocol, /BROWSER_GATEWAY_ACCESS_MODE_V1_LOCAL_DEVELOPMENT = 3/);
});
