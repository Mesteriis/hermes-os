import { access, readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

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

async function main() {
	if (!(await exists(path.join(repoRoot, 'AGENTS.md')))) {
		failures.push('AGENTS.md is required at repository root');
	}

	await checkAdrFiles();
	await checkMigrations();
	await checkDockerBoundary();

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Architecture guard passed.');
}

await main();
