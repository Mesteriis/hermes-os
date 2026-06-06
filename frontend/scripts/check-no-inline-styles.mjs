import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';

const sourceRoots = ['src', 'static'];
const checkedExtensions = new Set([
	'.html',
	'.js',
	'.jsx',
	'.mjs',
	'.svelte',
	'.ts',
	'.tsx'
]);
const forbiddenStylePatterns = [
	{
		pattern: /\sstyle\s*=/,
		message: 'inline style attributes are forbidden; move styles to CSS files'
	},
	{
		pattern: /<style(\s|>)/i,
		message: 'embedded style blocks are forbidden; move styles to CSS files'
	}
];

async function collectSourceFiles(root) {
	const entries = await readdir(root, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(root, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectSourceFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && checkedExtensions.has(path.extname(entry.name))) {
			files.push(entryPath);
		}
	}

	return files;
}

async function findStyleContractViolations() {
	const files = (await Promise.all(sourceRoots.map(collectSourceFiles))).flat();
	const violations = [];

	for (const file of files) {
		const source = await readFile(file, 'utf8');
		const lines = source.split('\n');

		for (const [index, line] of lines.entries()) {
			for (const { pattern, message } of forbiddenStylePatterns) {
				if (pattern.test(line)) {
					violations.push(`${file}:${index + 1}: ${message}`);
				}
			}
		}
	}

	return violations;
}

const violations = await findStyleContractViolations();

if (violations.length > 0) {
	console.error(violations.join('\n'));
	process.exit(1);
}
