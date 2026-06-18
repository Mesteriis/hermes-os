import { access, readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];
const selfTestMode = process.argv.includes('--self-test');
const boundaryBaselinePath = path.join(repoRoot, 'scripts', 'architecture-boundary-baseline.json');

const sharedBackendDomainModules = new Set(['api_support', 'settings']);
const businessBackendDomains = new Set([
	'agents',
	'calendar',
	'communications',
	'decisions',
	'documents',
	'graph',
	'knowledge',
	'mail',
	'notes',
	'obligations',
	'organizations',
	'personas',
	'persons',
	'projects',
	'radar',
	'relationships',
	'tasks',
	'timeline'
]);

async function exists(filePath) {
	try {
		await access(filePath);
		return true;
	} catch {
		return false;
	}
}

async function gitLsFiles() {
	const { stdout } = await execFileAsync('git', ['ls-files'], { cwd: repoRoot });
	return stdout.split('\n').filter(Boolean);
}

function normalizePath(filePath) {
	return filePath.split(path.sep).join('/');
}

async function collectFiles(relativeRoot, extensions) {
	const absoluteRoot = path.join(repoRoot, relativeRoot);
	let entries;
	try {
		entries = await readdir(absoluteRoot, { withFileTypes: true });
	} catch {
		return [];
	}

	const files = [];
	for (const entry of entries) {
		const relativePath = normalizePath(path.join(relativeRoot, entry.name));
		if (entry.isDirectory()) {
			files.push(...await collectFiles(relativePath, extensions));
			continue;
		}
		if (entry.isFile() && extensions.has(path.extname(entry.name))) {
			files.push(relativePath);
		}
	}
	return files;
}

function topLevelRustUseGroupItems(groupBody) {
	const items = [];
	let depth = 0;
	let segment = '';

	for (const char of groupBody) {
		if (char === '{') {
			depth += 1;
			segment += char;
			continue;
		}
		if (char === '}') {
			depth -= 1;
			segment += char;
			continue;
		}
		if (char === ',' && depth === 0) {
			items.push(segment.trim());
			segment = '';
			continue;
		}
		segment += char;
	}

	if (segment.trim() !== '') {
		items.push(segment.trim());
	}

	return items;
}

function extractGroupedBackendDomainImports(source) {
	const imports = new Set();
	const marker = 'crate::domains::{';
	let searchFrom = 0;

	while (searchFrom < source.length) {
		const markerIndex = source.indexOf(marker, searchFrom);
		if (markerIndex === -1) break;

		const groupStart = markerIndex + marker.length;
		let depth = 1;
		let cursor = groupStart;
		while (cursor < source.length && depth > 0) {
			const char = source[cursor];
			if (char === '{') depth += 1;
			if (char === '}') depth -= 1;
			cursor += 1;
		}

		if (depth !== 0) {
			searchFrom = groupStart;
			continue;
		}

		const groupBody = source.slice(groupStart, cursor - 1);
		for (const item of topLevelRustUseGroupItems(groupBody)) {
			const match = /^([a-zA-Z_][a-zA-Z0-9_]*)/.exec(item);
			if (match !== null && match[1] !== 'self' && match[1] !== 'super') {
				imports.add(match[1]);
			}
		}

		searchFrom = cursor;
	}

	return imports;
}

function extractBackendDomainImports(source) {
	const imports = extractGroupedBackendDomainImports(source);
	const directPattern = /\bcrate::domains::([a-zA-Z_][a-zA-Z0-9_]*)\b/g;
	for (const match of source.matchAll(directPattern)) {
		imports.add(match[1]);
	}
	return imports;
}

function backendBoundaryViolations(relativePath, source) {
	const violations = [];
	const domainMatch = /^backend\/src\/domains\/([^/]+)\//.exec(relativePath);
	const integrationMatch = /^backend\/src\/integrations\/([^/]+)\//.exec(relativePath);

	for (const importedDomain of extractBackendDomainImports(source)) {
		if (sharedBackendDomainModules.has(importedDomain)) continue;

		if (domainMatch !== null) {
			const currentDomain = domainMatch[1];
			if (importedDomain !== currentDomain) {
				violations.push({
					file: relativePath,
					importedDomain,
					message: `${relativePath}: domain "${currentDomain}" imports domain "${importedDomain}"; publish/consume events instead`
				});
			}
			continue;
		}

		if (integrationMatch !== null && businessBackendDomains.has(importedDomain)) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: integration "${integrationMatch[1]}" imports business domain "${importedDomain}"; publish integration/communication events instead`
			});
		}
	}

	return violations;
}

function resolveFrontendImport(relativePath, specifier) {
	if (specifier.startsWith('.')) {
		return normalizePath(path.posix.normalize(path.posix.join(path.posix.dirname(relativePath), specifier)));
	}
	if (specifier.startsWith('@/')) {
		return normalizePath(path.posix.normalize(path.posix.join('frontend/src', specifier.slice(2))));
	}
	if (specifier.startsWith('src/')) {
		return normalizePath(path.posix.normalize(path.posix.join('frontend', specifier)));
	}
	return null;
}

function frontendBoundaryViolations(relativePath, source) {
	const domainMatch = /^frontend\/src\/domains\/([^/]+)\//.exec(relativePath);
	if (domainMatch === null) return [];

	const currentDomain = domainMatch[1];
	const violations = [];
	const importPattern = /\bfrom\s+['"]([^'"]+)['"]|\bimport\s+['"]([^'"]+)['"]/g;

	for (const match of source.matchAll(importPattern)) {
		const specifier = match[1] ?? match[2];
		const resolved = resolveFrontendImport(relativePath, specifier);
		if (resolved === null) continue;

		const importedDomainMatch = /^frontend\/src\/domains\/([^/]+)\//.exec(resolved);
		if (importedDomainMatch === null) continue;

		const importedDomain = importedDomainMatch[1];
		if (importedDomain !== currentDomain) {
			violations.push({
				file: relativePath,
				importedDomain,
				message: `${relativePath}: frontend domain "${currentDomain}" imports domain "${importedDomain}"; compose domains from frontend/src/app instead`
			});
		}
	}

	return violations;
}

async function checkAdrFiles() {
	const adrDir = path.join(repoRoot, 'docs', 'adr');
	const entries = await readdir(adrDir);
	const adrFiles = entries.filter((entry) => entry.startsWith('ADR-') && entry.endsWith('.md'));
	const adrNumbers = new Map();

	for (const file of adrFiles) {
		const match = /^ADR-(\d{4})-[a-z0-9-]+\.md$/.exec(file);
		if (match === null) {
			failures.push(`docs/adr/${file}: ADR filename must be ADR-NNNN-kebab-case.md`);
			continue;
		}

		const number = match[1];
		const existing = adrNumbers.get(number);
		if (existing !== undefined) {
			failures.push(`docs/adr/${file}: duplicates ADR-${number} already used by ${existing}`);
		}
		adrNumbers.set(number, file);
	}

	for (const file of adrFiles) {
		const content = await readFile(path.join(adrDir, file), 'utf8');
		const statusMatch = /^Status:\s*(.+)$/m.exec(content);
		if (statusMatch === null) {
			failures.push(`docs/adr/${file}: missing Status line`);
		} else if (!/^(Proposed|Accepted|Temporary|Superseded|Superseded by ADR-\d{4})$/.test(statusMatch[1].trim())) {
			failures.push(`docs/adr/${file}: unsupported status "${statusMatch[1].trim()}"`);
		}

		for (const reference of content.matchAll(/\bADR-(\d{4})\b/g)) {
			if (!adrNumbers.has(reference[1])) {
				failures.push(`docs/adr/${file}: references missing ADR-${reference[1]}`);
			}
		}
	}
}

async function checkMigrations() {
	const migrationsDir = path.join(repoRoot, 'backend', 'migrations');
	const entries = await readdir(migrationsDir);
	const migrationFiles = entries.filter((entry) => entry.endsWith('.sql'));
	const seenNumbers = new Map();

	for (const file of migrationFiles) {
		const match = /^(\d{4})_[a-z0-9_]+\.sql$/.exec(file);
		if (match === null) {
			failures.push(`backend/migrations/${file}: migration filename must be NNNN_snake_case.sql`);
			continue;
		}

		const number = match[1];
		const existing = seenNumbers.get(number);
		if (existing !== undefined) {
			failures.push(`backend/migrations/${file}: duplicates migration number ${number} used by ${existing}`);
		}
		seenNumbers.set(number, file);
	}
}

async function checkDockerBoundary() {
	const trackedFiles = await gitLsFiles();
	for (const file of trackedFiles) {
		if ((file.endsWith('/Dockerfile') || file === 'Dockerfile') && !file.startsWith('docker/')) {
			failures.push(`${file}: Dockerfiles must stay under docker/`);
		}

		if (/docker-compose\.ya?ml$/.test(file) && !file.startsWith('docker/')) {
			failures.push(`${file}: Compose files must stay under docker/`);
		}

		if (file.startsWith('docker/data/') && file !== 'docker/data/.gitkeep') {
			failures.push(`${file}: docker/data contents are local state and must not be committed`);
		}
	}
}

function boundaryKey(violation) {
	return `${violation.file} -> ${violation.importedDomain}`;
}

function baselineKey(entry) {
	return `${entry.file} -> ${entry.importedDomain}`;
}

async function loadBoundaryBaseline() {
	const content = await readFile(boundaryBaselinePath, 'utf8');
	const baseline = JSON.parse(content);
	for (const section of ['backend', 'frontend']) {
		if (!Array.isArray(baseline[section])) {
			failures.push(`scripts/architecture-boundary-baseline.json: ${section} must be an array`);
			baseline[section] = [];
			continue;
		}

		for (const entry of baseline[section]) {
			if (
				entry === null ||
				typeof entry !== 'object' ||
				typeof entry.file !== 'string' ||
				typeof entry.importedDomain !== 'string'
			) {
				failures.push(
					`scripts/architecture-boundary-baseline.json: ${section} entries must contain file and importedDomain strings`
				);
			}
		}
	}
	return baseline;
}

function boundaryBaselineFailures(section, violations, baselineEntries) {
	const baselineKeys = new Set(baselineEntries.map(baselineKey));
	const currentKeys = new Set(violations.map(boundaryKey));
	const errors = [];

	for (const violation of violations) {
		if (!baselineKeys.has(boundaryKey(violation))) {
			errors.push(violation.message);
		}
	}

	for (const entry of baselineEntries) {
		const key = baselineKey(entry);
		if (!currentKeys.has(key)) {
			errors.push(
				`scripts/architecture-boundary-baseline.json: stale ${section} baseline entry ${key}`
			);
		}
	}

	return errors;
}

function applyBoundaryBaseline(section, violations, baselineEntries) {
	failures.push(...boundaryBaselineFailures(section, violations, baselineEntries));
}

async function checkLayerBoundaries() {
	const baseline = await loadBoundaryBaseline();
	const backendFiles = await collectFiles('backend/src', new Set(['.rs']));
	const frontendFiles = await collectFiles('frontend/src/domains', new Set(['.ts', '.vue']));
	const backendViolations = [];
	const frontendViolations = [];

	for (const file of backendFiles) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		backendViolations.push(...backendBoundaryViolations(file, source));
	}

	for (const file of frontendFiles) {
		const source = await readFile(path.join(repoRoot, file), 'utf8');
		frontendViolations.push(...frontendBoundaryViolations(file, source));
	}

	applyBoundaryBaseline('backend', backendViolations, baseline.backend);
	applyBoundaryBaseline('frontend', frontendViolations, baseline.frontend);
}

function assertSelfTest(name, condition) {
	if (!condition) {
		throw new Error(`architecture guard self-test failed: ${name}`);
	}
}

function runSelfTests() {
	assertSelfTest(
		'domain-to-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/radar/example.rs',
			'use crate::domains::persons::core::Person;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-business-domain backend import fails',
		backendBoundaryViolations(
			'backend/src/integrations/slack/client.rs',
			'use crate::domains::tasks::api::TaskStore;'
		).length === 1
	);
	assertSelfTest(
		'integration-to-business-domain grouped backend import fails',
		backendBoundaryViolations(
			'backend/src/integrations/slack/client.rs',
			'use crate::domains::{tasks::api::TaskStore, personas::core::Persona};'
		).length === 2
	);
	assertSelfTest(
		'handler-to-handler backend import fails',
		backendBoundaryViolations(
			'backend/src/domains/radar/api.rs',
			'use crate::domains::tasks::api::handlers::create_task;'
		).length === 1
	);
	assertSelfTest(
		'backend event platform import passes',
		backendBoundaryViolations(
			'backend/src/domains/tasks/example.rs',
			'use crate::platform::events::NewEventEnvelope;'
		).length === 0
	);
	assertSelfTest(
		'same backend domain import passes',
		backendBoundaryViolations(
			'backend/src/domains/radar/example.rs',
			'use crate::domains::radar::core::SignalStore;'
		).length === 0
	);
	assertSelfTest(
		'frontend domain-to-domain relative import fails',
		frontendBoundaryViolations(
			'frontend/src/domains/radar/views/RadarPage.vue',
			"import TasksPage from '../../tasks/views/TasksPage.vue'"
		).length === 1
	);
	assertSelfTest(
		'frontend domain-to-domain alias import fails',
		frontendBoundaryViolations(
			'frontend/src/domains/radar/views/RadarPage.vue',
			"import TasksPage from '@/domains/tasks/views/TasksPage.vue'"
		).length === 1
	);
	assertSelfTest(
		'frontend app composition is outside domain lint',
		frontendBoundaryViolations(
			'frontend/src/app/views/HomeView.vue',
			"import TasksPage from '../../domains/tasks/views/TasksPage.vue'"
		).length === 0
	);
	assertSelfTest(
		'exact baseline allows only the listed legacy pair',
		boundaryBaselineFailures(
			'backend',
			[
				{
					file: 'backend/src/domains/mail/handlers/mod.rs',
					importedDomain: 'tasks',
					message: 'known legacy tasks import'
				}
			],
			[{ file: 'backend/src/domains/mail/handlers/mod.rs', importedDomain: 'tasks' }]
		).length === 0
	);
	assertSelfTest(
		'exact baseline rejects new domain import in legacy file',
		boundaryBaselineFailures(
			'backend',
			[
				{
					file: 'backend/src/domains/mail/handlers/mod.rs',
					importedDomain: 'tasks',
					message: 'known legacy tasks import'
				},
				{
					file: 'backend/src/domains/mail/handlers/mod.rs',
					importedDomain: 'radar',
					message: 'new radar import'
				}
			],
			[{ file: 'backend/src/domains/mail/handlers/mod.rs', importedDomain: 'tasks' }]
		).length === 1
	);
	assertSelfTest(
		'exact baseline reports stale entries',
		boundaryBaselineFailures(
			'frontend',
			[],
			[{ file: 'frontend/src/domains/telegram/api/telegram.ts', importedDomain: 'communications' }]
		).length === 1
	);
	console.log('Architecture guard self-tests passed.');
}

async function main() {
	if (selfTestMode) {
		runSelfTests();
		return;
	}

	if (!(await exists(path.join(repoRoot, 'AGENTS.md')))) {
		failures.push('AGENTS.md is required at repository root');
	}

	await checkAdrFiles();
	await checkMigrations();
	await checkDockerBoundary();
	await checkLayerBoundaries();

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Architecture guard passed.');
}

await main();
