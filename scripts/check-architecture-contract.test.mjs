import { access, readFile } from 'node:fs/promises';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

async function exists(relativePath) {
	try {
		await access(path.join(repoRoot, relativePath));
		return true;
	} catch {
		return false;
	}
}

const contractPath = 'scripts/architecture-contract.json';
const contract = JSON.parse(await readFile(path.join(repoRoot, contractPath), 'utf8'));

assert.equal(
	await exists('scripts/architecture-boundary-baseline.json'),
	false,
	'architecture-boundary-baseline.json must not exist'
);

assert.equal(contract.schema_version, 1, 'architecture contract schema_version must be 1');
assert.ok(Array.isArray(contract.interaction_kinds), 'architecture contract must list interaction kinds');
assert.deepEqual(
	contract.interaction_kinds,
	['direct_call', 'command_port', 'query_port', 'event', 'projection', 'runtime_integration_api'],
	'architecture contract interaction kinds are the public communication vocabulary'
);

assert.ok(contract.backend?.layers?.domains?.deny, 'backend domain deny rules must be explicit');
assert.ok(contract.backend.layers.domains.deny.includes('other_domains'));
assert.ok(contract.backend.layers.domains.deny.includes('integrations'));
assert.ok(contract.backend.layers.integrations.deny.includes('domains'));
assert.ok(contract.backend.layers.workflows.allow.includes('domain_command_ports'));
assert.ok(contract.backend.layers.workflows.allow.includes('domain_query_ports'));
assert.ok(contract.backend.layers.app.deny.includes('stores'));
assert.ok(contract.backend.layers.ai.deny.includes('domain_stores'));

assert.ok(contract.frontend?.layers?.domains?.deny.includes('other_frontend_domains'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('telegram'));
assert.ok(contract.frontend.provider_business_cache_roots.forbidden.includes('whatsapp'));
assert.ok(contract.frontend.provider_business_cache_roots.allowed_business_root === 'communications');

console.log('Architecture contract tests passed.');
