import { readFile } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dependencyKinds = ['normal', 'build', 'dev'];

export function evaluateWorkspacePolicy({ packages, dependencies, roles, roleEdges }) {
	const failures = [];
	const workspacePackages = [...packages.values()].filter((pkg) => pkg.workspace);
	const workspaceByName = new Map(workspacePackages.map((pkg) => [pkg.name, pkg]));
	const rolesByPackageId = new Map();

	for (const [roleName, role] of Object.entries(roles)) {
		for (const packageName of role.packages ?? []) {
			const pkg = workspaceByName.get(packageName);
			if (!pkg) {
				failures.push(`role ${roleName} references missing workspace package "${packageName}"`);
				continue;
			}
			const assignedRoles = rolesByPackageId.get(pkg.id) ?? [];
			assignedRoles.push(roleName);
			rolesByPackageId.set(pkg.id, assignedRoles);
		}
	}

	for (const pkg of workspacePackages) {
		const assignedRoles = rolesByPackageId.get(pkg.id) ?? [];
		if (assignedRoles.length === 0) {
			failures.push(`workspace package "${pkg.name}" has no role`);
		} else if (assignedRoles.length > 1) {
			failures.push(
				`workspace package "${pkg.name}" has multiple roles: ${assignedRoles.join(', ')}`
			);
		}
	}

	if (failures.length > 0) return failures;

	const roleByPackageId = new Map(
		workspacePackages.map((pkg) => [pkg.id, rolesByPackageId.get(pkg.id)[0]])
	);

	for (const pkg of workspacePackages) {
		const sourceRole = roleByPackageId.get(pkg.id);
		const allowedByKind = roleEdges[sourceRole];
		if (!allowedByKind) {
			failures.push(`role ${sourceRole} has no dependency policy`);
			continue;
		}

		for (const dependency of dependencies.get(pkg.id) ?? []) {
			const targetPackage = packages.get(dependency.packageId);
			if (!targetPackage?.workspace) continue;
			const targetRole = roleByPackageId.get(targetPackage.id);
			const kind = normalizeDependencyKind(dependency.kind);
			const allowedRoles = allowedByKind[kind];
			if (!Array.isArray(allowedRoles)) {
				failures.push(`role ${sourceRole} has no ${kind} dependency policy`);
				continue;
			}
			if (!allowedRoles.includes(targetRole)) {
				failures.push(
					`${pkg.name}: role ${sourceRole} cannot depend on role ${targetRole} via ${kind} dependency`
				);
			}
		}
	}

	for (const pkg of workspacePackages) {
		const roleName = roleByPackageId.get(pkg.id);
		const forbiddenDependencies = roles[roleName]?.forbidden_dependencies ?? [];
		const closure = collectDependencyClosure(pkg.id, dependencies);
		for (const forbiddenDependency of forbiddenDependencies) {
			if ([...closure].some((packageId) => packages.get(packageId)?.name === forbiddenDependency)) {
				failures.push(
					`${pkg.name}: forbidden dependency "${forbiddenDependency}" for role ${roleName}`
				);
			}
		}
	}

	return failures;
}

function normalizeDependencyKind(kind) {
	return kind === 'build' || kind === 'dev' ? kind : 'normal';
}

function collectDependencyClosure(packageId, dependencies) {
	const seen = new Set();
	const pending = (dependencies.get(packageId) ?? []).map((dependency) => dependency.packageId);

	while (pending.length > 0) {
		const dependencyId = pending.pop();
		if (seen.has(dependencyId)) continue;
		seen.add(dependencyId);
		for (const dependency of dependencies.get(dependencyId) ?? []) {
			pending.push(dependency.packageId);
		}
	}

	return seen;
}

async function workspaceMetadata() {
	const { stdout } = await execFileAsync(
		'cargo',
		['metadata', '--locked', '--format-version', '1'],
		{ cwd: repoRoot, maxBuffer: 32 * 1024 * 1024 }
	);
	const metadata = JSON.parse(stdout);
	const workspaceIds = new Set(metadata.workspace_members);
	const packages = new Map(
		metadata.packages.map((pkg) => [pkg.id, { id: pkg.id, name: pkg.name, workspace: workspaceIds.has(pkg.id) }])
	);
	const dependencies = new Map();

	for (const node of metadata.resolve?.nodes ?? []) {
		const edges = [];
		for (const dependency of node.deps) {
			for (const dependencyKind of dependency.dep_kinds ?? [{ kind: null }]) {
				edges.push({
					packageId: dependency.pkg,
					kind: normalizeDependencyKind(dependencyKind.kind)
				});
			}
		}
		dependencies.set(node.id, edges);
	}

	return { packages, dependencies };
}

async function main() {
	const contract = JSON.parse(
		await readFile(path.join(repoRoot, 'scripts', 'architecture-contract.json'), 'utf8')
	);
	const { packages, dependencies } = await workspaceMetadata();
	const failures = evaluateWorkspacePolicy({
		packages,
		dependencies,
		roles: contract.workspace?.roles ?? {},
		roleEdges: contract.workspace?.role_edges ?? {}
	});

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Cargo architecture guard passed.');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	await main();
}
