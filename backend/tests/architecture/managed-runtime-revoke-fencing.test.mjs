import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const BACKEND_ROOT = new URL('../..', import.meta.url);
const OWNER_CONTROL_DISPATCH = new URL(
  'src/kernel/src/identity/owner_control/dispatch.rs',
  BACKEND_ROOT,
);
const MANAGED_LAUNCH = new URL(
  'src/kernel/src/platform/macos/managed_launch.rs',
  BACKEND_ROOT,
);
const MANAGED_FENCE_CONSUMERS = [
  'src/kernel/src/modules/capability/router.rs',
  'src/kernel/src/platform/blob/session.rs',
  'src/kernel/src/platform/events/credential/handler.rs',
  'src/kernel/src/platform/vault/managed_route.rs',
  'src/kernel/src/platform/vault/owner_derived_key.rs',
  'src/kernel/src/platform/vault/provider_credential.rs',
].map((path) => new URL(path, BACKEND_ROOT));

test('owner suspend or revoke durably fences grants before stopping a managed runtime', async () => {
  const source = await readFile(OWNER_CONTROL_DISPATCH, 'utf8');
  const transitionStart = source.indexOf('fn transition(');
  const transitionEnd = source.indexOf('\nfn begin(', transitionStart);
  const transition = source.slice(transitionStart, transitionEnd);

  assert.match(
    source,
    /Operation::TransitionModuleRegistration\(request\) =>\s*(?:\{\s*)?transition\(store, supervisor, sessions, request\)/,
  );
  assert.match(transition, /module_registry::transition_after_owner_authorization/);
  assert.match(transition, /supervisor\.stop_if_active\(registration\.registration_id\(\)\)/);
  assert.ok(
    transition.indexOf('transition_after_owner_authorization')
      < transition.indexOf('stop_if_active'),
    'the durable grant-epoch fence must precede process stop',
  );
});

test('managed runtime generation advances from the persisted high-watermark', async () => {
  const source = await readFile(MANAGED_LAUNCH, 'utf8');
  const generationStart = source.indexOf('fn next_runtime_generation(');
  const generation = source.slice(generationStart);

  assert.match(generation, /managed_launch_generation_high_watermark/);
  assert.doesNotMatch(generation, /effective_managed_launch_record/);
});

test('managed release replacement durably changes binding before stopping the old runtime', async () => {
  const source = await readFile(OWNER_CONTROL_DISPATCH, 'utf8');
  const bindingStart = source.indexOf('fn bind_managed_release(');
  const bindingEnd = source.indexOf('\nfn start_managed_runtime(', bindingStart);
  const binding = source.slice(bindingStart, bindingEnd);

  assert.match(
    source,
    /bind_managed_release\(store, supervisor, sessions, request\)/,
  );
  assert.match(binding, /bind_current_installed_release/);
  assert.match(binding, /supervisor\.stop_if_active\(binding\.registration_id\(\)\)/);
  assert.ok(
    binding.indexOf('bind_current_installed_release')
      < binding.indexOf('stop_if_active'),
    'the durable binding replacement must precede process stop',
  );
});

test('all managed module data-plane routes share the exact current binding fence', async () => {
  const sources = await Promise.all(
    MANAGED_FENCE_CONSUMERS.map((path) => readFile(path, 'utf8')),
  );

  for (const source of sources) {
    assert.match(source, /current_managed_runtime_matches/);
  }
});
