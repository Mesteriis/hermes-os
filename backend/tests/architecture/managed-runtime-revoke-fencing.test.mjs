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
