import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const PROJECT_ROOT = new URL('../../../', import.meta.url);
const INVENTORY_PATH = new URL(
  'architecture/communications-settings-reconstruction.json',
  BACKEND_ROOT,
);
const POLICY_PATH = new URL('architecture/policy.json', BACKEND_ROOT);
const ADR_PATH = new URL(
  'docs/adr/ADR-0282-full-communications-and-settings-capability-reconstruction.md',
  PROJECT_ROOT,
);
const TELEGRAM_AUTOMATION_ADR_PATH = new URL(
  'docs/adr/ADR-0283-telegram-automation-management-and-preview-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_CALLS_ADR_PATH = new URL(
  'docs/adr/ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_REALTIME_ADR_PATH = new URL(
  'docs/adr/ADR-0287-telegram-operational-realtime-replay-boundary.md',
  PROJECT_ROOT,
);
const TELEGRAM_CLIENT_CONTRACT_PATH = new URL(
  'src/telegram-api/src/client_contract.rs',
  BACKEND_ROOT,
);
const TELEGRAM_RUNTIME_ADMISSION_PATH = new URL(
  'src/telegram-runtime/src/admission.rs',
  BACKEND_ROOT,
);
const TELEGRAM_MANAGED_FLOW_PATH = new URL(
  'tests/support/kernel-recovery/src/tests/managed_storage_vault_docker/telegram_managed_flow.rs',
  BACKEND_ROOT,
);
const WHATSAPP_OPERATIONAL_ADR_PATH = new URL(
  'docs/adr/ADR-0286-whatsapp-operational-read-and-realtime-boundary.md',
  PROJECT_ROOT,
);

const ALLOWED_ROLES = new Set(['app', 'domain', 'engine', 'integration', 'platform', 'workflow']);
const ALLOWED_STATES = new Set(['implemented', 'planned']);
const BUSINESS_OWNER_ROLES = new Set(['domain', 'engine', 'integration', 'workflow']);
const FORBIDDEN_BUSINESS_OWNERS = new Set(['core', 'gateway', 'kernel', 'settings']);

test('ADR-0282 keeps an exact incomplete reconstruction inventory', async () => {
  const [inventorySource, policySource, adrSource] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(POLICY_PATH, 'utf8'),
    readFile(ADR_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const policy = JSON.parse(policySource);

  assert.equal(inventory.version, 1);
  assert.equal(inventory.adr, 'ADR-0282');
  assert.equal(inventory.status, 'incomplete');
  assert.equal(inventory.completionGate, 'communications_settings_reconstruction_complete_v1');
  assert.equal(inventory.legacyAuthorityAllowed, false);
  assert.ok(inventory.slices.length > 20);

  const gates = inventory.slices.map(({ gate }) => gate);
  assert.equal(new Set(gates).size, gates.length, 'reconstruction gates must be unique');
  assert.ok(inventory.slices.every(({ state }) => ALLOWED_STATES.has(state)));
  assert.ok(inventory.slices.some(({ state }) => state === 'planned'));

  for (const slice of inventory.slices) {
    assert.ok(ALLOWED_ROLES.has(slice.role), `unknown owner role for ${slice.gate}`);
    assert.ok(slice.owner.length > 0, `missing owner for ${slice.gate}`);
    assert.ok(Array.isArray(slice.dependsOn), `missing dependencies for ${slice.gate}`);
    assert.match(adrSource, new RegExp(`\\b${slice.gate}\\b`), `${slice.gate} is absent from ADR-0282`);
    if (BUSINESS_OWNER_ROLES.has(slice.role)) {
      assert.ok(
        !FORBIDDEN_BUSINESS_OWNERS.has(slice.owner),
        `${slice.gate} assigns business behavior to ${slice.owner}`,
      );
    }
  }

  const activeCapabilities = new Set(policy.implementation.ownerInventory.businessCapabilities);
  const knownDependencies = new Set([
    ...activeCapabilities,
    ...Object.keys(policy.phaseGates.requires),
    ...gates,
  ]);
  for (const slice of inventory.slices) {
    for (const dependency of slice.dependsOn) {
      assert.ok(
        knownDependencies.has(dependency),
        `${slice.gate} has an unknown dependency ${dependency}`,
      );
    }
  }
  assert.ok(
    gates.every((gate) => !activeCapabilities.has(gate)),
    'reconstruction slice must not be active before an exact production admission gate',
  );
  assert.ok(
    !Object.hasOwn(policy.phaseGates.requires, inventory.completionGate),
    'completion gate must remain closed while inventory is incomplete',
  );
});

test('provider operational slices remain separate integrations', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));
  const providerSlices = new Map(
    inventory.slices
      .filter(({ gate }) => gate.endsWith('_full_operational_v1'))
      .map((slice) => [slice.owner, slice]),
  );

  assert.deepEqual([...providerSlices.keys()].sort(), ['telegram', 'whatsapp', 'zulip']);
  assert.ok([...providerSlices.values()].every(({ role }) => role === 'integration'));

  for (const owner of ['mail', 'telegram', 'whatsapp', 'zulip']) {
    const ownerSlices = inventory.slices.filter((slice) => slice.owner === owner);
    assert.ok(ownerSlices.length > 0, `${owner} must have an independent reconstruction slice`);
    assert.ok(ownerSlices.every(({ role }) => role === 'integration'));
  }
});

test('Telegram completion remains closed behind its independent capability slices', async () => {
  const [
    inventorySource,
    automationAdrSource,
    callsAdrSource,
    realtimeAdrSource,
    clientContractSource,
    runtimeAdmissionSource,
    managedFlowSource,
  ] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(TELEGRAM_AUTOMATION_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_CALLS_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_REALTIME_ADR_PATH, 'utf8'),
    readFile(TELEGRAM_CLIENT_CONTRACT_PATH, 'utf8'),
    readFile(TELEGRAM_RUNTIME_ADMISSION_PATH, 'utf8'),
    readFile(TELEGRAM_MANAGED_FLOW_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const telegramSlices = new Map(
    inventory.slices
      .filter(({ owner }) => owner === 'telegram')
      .map((slice) => [slice.gate, slice]),
  );
  const requiredTelegramGates = [
    'telegram_automation_v1',
    'telegram_call_history_v1',
    'telegram_call_media_v1',
    'telegram_call_signaling_v1',
    'telegram_calls_operational_v1',
    'telegram_core_operational_v1',
    'telegram_folder_reassignment_v1',
    'telegram_runtime_reconfiguration_v1',
  ];
  const fullGate = telegramSlices.get('telegram_full_operational_v1');

  assert.deepEqual(
    [...telegramSlices.keys()].filter((gate) => gate !== 'telegram_full_operational_v1').sort(),
    requiredTelegramGates,
  );
  assert.deepEqual(
    [...fullGate.dependsOn].sort(),
    requiredTelegramGates.filter((gate) => !gate.startsWith('telegram_call_')),
  );
  assert.ok([...telegramSlices.values()].every(({ role }) => role === 'integration'));

  const automationGate = telegramSlices.get('telegram_automation_v1');
  assert.equal(automationGate.state, 'implemented');
  assert.equal(telegramSlices.get('telegram_core_operational_v1').state, 'implemented');
  assert.equal(fullGate.state, 'planned');
  assert.equal(telegramSlices.get('telegram_calls_operational_v1').state, 'planned');
  assert.equal(telegramSlices.get('telegram_call_signaling_v1').state, 'implemented');
  assert.deepEqual(automationGate.dependsOn, ['telegram_core_operational_v1']);
  assert.match(automationAdrSource, /hermes-telegram-automation-api/);
  assert.match(automationAdrSource, /hermes-telegram-automation-core/);
  assert.match(automationAdrSource, /hermes-telegram-automation-persistence/);
  assert.match(automationAdrSource, /telegram\.automation\.query\.v1/);
  assert.match(automationAdrSource, /telegram\.automation\.command\.v1/);
  assert.match(automationAdrSource, /telegram_automation_execution_v1/);

  const callsGate = telegramSlices.get('telegram_calls_operational_v1');
  assert.deepEqual([...callsGate.dependsOn].sort(), [
    'telegram_call_history_v1',
    'telegram_call_media_v1',
    'telegram_call_signaling_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_history_v1').dependsOn, [
    'telegram_core_operational_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_signaling_v1').dependsOn, [
    'telegram_call_history_v1',
  ]);
  assert.deepEqual(telegramSlices.get('telegram_call_media_v1').dependsOn, [
    'telegram_call_signaling_v1',
  ]);
  assert.match(callsAdrSource, /hermes-telegram-calls-api/);
  assert.match(callsAdrSource, /hermes-telegram-calls-core/);
  assert.match(callsAdrSource, /hermes-telegram-calls-persistence/);
  assert.match(callsAdrSource, /hermes-telegram-call-media-contract/);
  assert.match(callsAdrSource, /hermes-telegram-call-media-tgcalls/);
  assert.match(callsAdrSource, /telegram\.calls\.query\.v1/);
  assert.match(callsAdrSource, /telegram\.calls\.command\.v1/);
  assert.match(callsAdrSource, /telegram\.calls\.realtime\.v1/);
  assert.match(callsAdrSource, /call\.id.*непостоянным/);
  assert.match(callsAdrSource, /fixture PCM[\s\S]*не закрывают production admission/);

  assert.match(realtimeAdrSource, /telegram\.realtime\.v1/);
  assert.match(realtimeAdrSource, /reset_required/);
  assert.match(realtimeAdrSource, /Состояние реализации: Реализовано/);
  assert.match(clientContractSource, /TelegramClientContractV1[\s\S]*Realtime/);
  assert.match(clientContractSource, /TELEGRAM_CLIENT_CONTRACT_REVISION: u32 = 4/);
  assert.match(runtimeAdmissionSource, /TelegramClientContractV1::Realtime/);
  assert.match(
    managedFlowSource,
    /managed_telegram_core_operational_projection_is_restart_safe/,
  );
  assert.match(managedFlowSource, /managed_telegram_realtime_route_requires_exact_grant/);
});

test('WhatsApp completion remains closed behind independent read and realtime slices', async () => {
  const [inventorySource, whatsappAdrSource] = await Promise.all([
    readFile(INVENTORY_PATH, 'utf8'),
    readFile(WHATSAPP_OPERATIONAL_ADR_PATH, 'utf8'),
  ]);
  const inventory = JSON.parse(inventorySource);
  const whatsappSlices = new Map(
    inventory.slices
      .filter(({ owner }) => owner === 'whatsapp')
      .map((slice) => [slice.gate, slice]),
  );

  assert.deepEqual([...whatsappSlices.keys()].sort(), [
    'whatsapp_full_operational_v1',
    'whatsapp_integration_v1',
    'whatsapp_operational_read_v1',
    'whatsapp_operational_realtime_v1',
  ]);
  assert.ok([...whatsappSlices.values()].every(({ role }) => role === 'integration'));
  assert.equal(whatsappSlices.get('whatsapp_integration_v1').state, 'implemented');
  assert.equal(whatsappSlices.get('whatsapp_full_operational_v1').state, 'planned');
  assert.equal(whatsappSlices.get('whatsapp_operational_read_v1').state, 'implemented');
  assert.equal(whatsappSlices.get('whatsapp_operational_realtime_v1').state, 'implemented');
  assert.deepEqual(whatsappSlices.get('whatsapp_operational_read_v1').dependsOn, [
    'whatsapp_integration_v1',
  ]);
  assert.deepEqual(whatsappSlices.get('whatsapp_operational_realtime_v1').dependsOn, [
    'whatsapp_operational_read_v1',
  ]);
  assert.deepEqual(
    [...whatsappSlices.get('whatsapp_full_operational_v1').dependsOn].sort(),
    [
      'client_gateway_v1',
      'nats_data_plane_v1',
      'whatsapp_operational_read_v1',
      'whatsapp_operational_realtime_v1',
    ],
  );

  assert.match(whatsappAdrSource, /whatsapp\.operational\.query\.v1/);
  assert.match(whatsappAdrSource, /whatsapp\.operational\.realtime\.v1/);
  assert.match(whatsappAdrSource, /hermes-whatsapp-api/);
  assert.match(whatsappAdrSource, /hermes-whatsapp-core/);
  assert.match(whatsappAdrSource, /hermes-whatsapp-persistence/);
  assert.match(whatsappAdrSource, /DDL-only/);
  assert.match(whatsappAdrSource, /Fake backfill запрещён/);
});

test('cross-owner and AI use cases are distinct workflow units', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));
  const workflowOwners = inventory.slices
    .filter(({ role }) => role === 'workflow')
    .map(({ owner }) => owner);

  assert.equal(new Set(workflowOwners).size, workflowOwners.length);
  assert.ok(workflowOwners.includes('communication_delivery_intent'));
  assert.ok(workflowOwners.includes('communication_reply_suggestion'));
  assert.ok(workflowOwners.includes('communication_translation'));
  assert.ok(workflowOwners.includes('communication_task_candidate_extraction'));
  assert.ok(workflowOwners.includes('call_transcription'));
  assert.ok(!workflowOwners.includes('communications'));
  assert.ok(!workflowOwners.includes('generic_ai'));
  assert.ok(!workflowOwners.includes('settings'));
});

test('historical presentation facades do not become admitted capabilities', async () => {
  const inventory = JSON.parse(await readFile(INVENTORY_PATH, 'utf8'));

  assert.deepEqual(inventory.historicalFacades, [
    'discord_channels',
    'google_meet_calls',
    'mattermost_channels',
    'microsoft_teams_calls',
    'phone_calls_without_admitted_provider',
    'slack_channels',
    'telemost_calls',
    'zoom_calls',
  ]);
  assert.ok(
    inventory.historicalFacades.every(
      (facade) => !inventory.slices.some(({ gate, owner }) => gate.includes(facade) || owner === facade),
    ),
  );
});
