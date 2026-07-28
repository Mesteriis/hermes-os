import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);

const paths = {
  policy: new URL('architecture/policy.json', BACKEND_ROOT),
  sourceManifest: new URL(
    'src/communications-evidence-export-source-api/Cargo.toml',
    BACKEND_ROOT,
  ),
  sourceContract: new URL(
    'src/communications-evidence-export-source-api/proto/hermes/communications/evidence_export_source/v1/evidence_export_source.proto',
    BACKEND_ROOT,
  ),
  communicationsManifest: new URL('src/communications-runtime/Cargo.toml', BACKEND_ROOT),
  communicationsAdmission: new URL(
    'src/communications-runtime/src/admission.rs',
    BACKEND_ROOT,
  ),
  workflowManifest: new URL('src/communications-export-runtime/Cargo.toml', BACKEND_ROOT),
  workflowAdmission: new URL(
    'src/communications-export-runtime/src/admission.rs',
    BACKEND_ROOT,
  ),
  workflowRuntime: new URL(
    'src/communications-export-runtime/src/main.rs',
    BACKEND_ROOT,
  ),
  workflowAssembly: new URL(
    'src/communications-export-assembly/Cargo.toml',
    BACKEND_ROOT,
  ),
  kernelLaunch: new URL('src/kernel/src/platform/macos/managed_launch.rs', BACKEND_ROOT),
  runtimeDescriptorValidation: new URL(
    'src/platform/runtime_protocol/src/validation/descriptor.rs',
    BACKEND_ROOT,
  ),
  controlStoreClientBlob: new URL(
    'src/kernel/control_store/sqlite/src/module_state/client_blob_route.rs',
    BACKEND_ROOT,
  ),
  controlStoreClientBlobSchema: new URL(
    'src/kernel/control_store/sqlite/src/schema/v42_to_v43.rs',
    BACKEND_ROOT,
  ),
  gatewayClientBlob: new URL(
    'src/api/gateway/runtime/src/browser/client_blob.rs',
    BACKEND_ROOT,
  ),
  ownerControl: new URL(
    'src/api/gateway/contracts/proto/hermes/gateway/v1/owner_control.proto',
    BACKEND_ROOT,
  ),
  devAssembly: new URL('development/assembly/src/main.rs', BACKEND_ROOT),
  release: new URL('scripts/materialize-dev-release.sh', BACKEND_ROOT),
  frontendRoute: new URL(
    'frontend/src/domains/communications/views/CanonicalCommunicationsRoute.vue',
    PROJECT_ROOT,
  ),
  frontendApp: new URL('frontend/src/app/layout/AppLayoutRoot.vue', PROJECT_ROOT),
  frontendWorkflow: new URL(
    'frontend/src/workflows/communications-export/api/communicationsEvidenceExport.ts',
    PROJECT_ROOT,
  ),
  adr: new URL(
    'docs/adr/ADR-0318-communications-evidence-export-workflow.md',
    PROJECT_ROOT,
  ),
};

test('Communications export is one exact workflow family with a public domain source port', async () => {
  const [
    policySource,
    sourceManifest,
    sourceContract,
    communicationsManifest,
    communicationsAdmission,
    workflowManifest,
    workflowAdmission,
    workflowAssembly,
    adr,
  ] = await Promise.all([
    readFile(paths.policy, 'utf8'),
    readFile(paths.sourceManifest, 'utf8'),
    readFile(paths.sourceContract, 'utf8'),
    readFile(paths.communicationsManifest, 'utf8'),
    readFile(paths.communicationsAdmission, 'utf8'),
    readFile(paths.workflowManifest, 'utf8'),
    readFile(paths.workflowAdmission, 'utf8'),
    readFile(paths.workflowAssembly, 'utf8'),
    readFile(paths.adr, 'utf8'),
  ]);
  const policy = JSON.parse(policySource);
  const sourceSchema = sourceContract.replaceAll(/\/\/.*$/gm, '');
  assert.equal(policy.implementation.currentSlice, 'communications_export_v1');
  assert.deepEqual(policy.implementation.ownerInventory.workflows, ['communications_export']);
  assert.deepEqual(
    policy.implementation.productionPackages
      .filter(({ owner }) => owner === 'communications_export')
      .map(({ name, surface }) => `${name}:${surface}`),
    [
      'hermes-communications-export-api:contract',
      'hermes-communications-export-core:implementation',
      'hermes-communications-export-persistence:persistence',
      'hermes-communications-export-runtime:runtime',
      'hermes-communications-export-assembly:assembly',
    ],
  );
  assert.match(sourceManifest, /role = "domain"[\s\S]*owner = "communications"/);
  assert.match(sourceSchema, /message PrepareEvidenceExportCommandV1/);
  assert.match(sourceSchema, /message EvidenceExportPreparedV1/);
  assert.doesNotMatch(sourceSchema, /\b(?:provider|locator|map)\b/i);
  assert.match(communicationsManifest, /hermes-communications-evidence-export-source-api/);
  assert.doesNotMatch(
    communicationsManifest,
    /hermes-communications-export-(?:api|core|persistence|runtime|assembly)/,
  );
  assert.match(communicationsAdmission, /communications\.export-source\.v1/);
  assert.match(workflowManifest, /role = "workflow"[\s\S]*owner = "communications_export"/);
  assert.doesNotMatch(
    workflowManifest,
    /hermes-(?:communications-domain|communications-persistence|mail|telegram|whatsapp|zulip)/,
  );
  assert.match(workflowAdmission, /ModuleKindV1::Workflow/);
  assert.match(workflowAssembly, /surface = "assembly"/);
  assert.match(adr, /hermes-communications-export-assembly/);
});

test('Kernel launches workflow through a distinct provider-neutral configuration', async () => {
  const [kernelLaunch, ownerControl, workflowRuntime, devAssembly, release] = await Promise.all([
    readFile(paths.kernelLaunch, 'utf8'),
    readFile(paths.ownerControl, 'utf8'),
    readFile(paths.workflowRuntime, 'utf8'),
    readFile(paths.devAssembly, 'utf8'),
    readFile(paths.release, 'utf8'),
  ]);
  assert.match(kernelLaunch, /fn start_reserved_workflow/);
  assert.match(kernelLaunch, /expected_module_kind: ModuleKindV1::Workflow/);
  const workflowLaunch = kernelLaunch.match(
    /pub\(crate\) fn start_reserved_workflow\([\s\S]*?\n\}/,
  )?.[0] ?? '';
  assert.doesNotMatch(workflowLaunch, /ManagedIntegration|ManagedDomain/);
  assert.match(workflowLaunch, /settings_snapshot_bytes: None/);
  assert.match(workflowLaunch, /host_bridge_configuration: None/);
  assert.match(ownerControl, /StartReservedWorkflowRuntimeRequestV1/);
  assert.match(workflowRuntime, /ManagedWorkflowRuntimeConfigurationV1/);
  assert.match(devAssembly, /ModuleRuntimeKindV1::Workflow/);
  assert.match(devAssembly, /start_reserved_workflow_runtime/);
  assert.match(release, /hermes-communications-export-assembly/);
  assert.match(release, /communications_export\.release-artifacts\.json/);
});

test('export artifact stays within one exact platform client Blob ceiling', async () => {
  const [
    runtimeDescriptorValidation,
    controlStoreClientBlob,
    controlStoreClientBlobSchema,
    gatewayClientBlob,
    workflowAdmission,
  ] = await Promise.all([
      readFile(paths.runtimeDescriptorValidation, 'utf8'),
      readFile(paths.controlStoreClientBlob, 'utf8'),
      readFile(paths.controlStoreClientBlobSchema, 'utf8'),
      readFile(paths.gatewayClientBlob, 'utf8'),
      readFile(paths.workflowAdmission, 'utf8'),
    ]);
  const exactCeiling = /const MAX_CLIENT_BLOB_RESPONSE_BYTES: u64 = 24 \* 1024 \* 1024;/;
  assert.match(runtimeDescriptorValidation, exactCeiling);
  assert.match(controlStoreClientBlob, exactCeiling);
  assert.match(
    controlStoreClientBlobSchema,
    /max_response_bytes BETWEEN 1 AND 25165824/,
  );
  assert.match(gatewayClientBlob, exactCeiling);
  assert.match(
    workflowAdmission,
    /max_response_bytes: COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1/,
  );
});

test('Frontend composition passes canonical IDs into a generated workflow without domain imports', async () => {
  const [route, app, workflow] = await Promise.all([
    readFile(paths.frontendRoute, 'utf8'),
    readFile(paths.frontendApp, 'utf8'),
    readFile(paths.frontendWorkflow, 'utf8'),
  ]);
  assert.match(route, /canonicalMessageSelected/);
  assert.doesNotMatch(route, /workflows\/communications-export|communicationsEvidenceExport/);
  assert.match(app, /CommunicationsEvidenceExportWorkflow/);
  assert.match(app, /communications\.export\.v1/);
  assert.match(workflow, /getCommunicationsExportCommandClient/);
  assert.match(workflow, /getCommunicationsExportQueryClient/);
  assert.match(workflow, /getCommunicationsExportTicketClient/);
  assert.match(workflow, /BrowserGatewayFetch/);
  assert.doesNotMatch(workflow, /integrations\/(?:mail|telegram|whatsapp|zulip)|provider/);
});
