import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';

const sourceRoot = path.resolve('src');
const warningLimit = 300;
const failureLimit = 500;
const criticalLimit = 700;
const maxReportedFiles = 20;

async function collectVueFiles(directory) {
	const entries = await readdir(directory, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(directory, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectVueFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && entry.name.endsWith('.vue')) {
			files.push(entryPath);
		}
	}

	return files;
}

function countLines(source) {
	return source.endsWith('\n') ? source.split('\n').length - 1 : source.split('\n').length;
}

function formatFileReport(files) {
	return files
		.slice(0, maxReportedFiles)
		.map(({ relativePath, lineCount }) => `- ${relativePath}: ${lineCount} lines`)
		.join('\n');
}

const vueFiles = await collectVueFiles(sourceRoot);
const componentLineCounts = [];

for (const filePath of vueFiles) {
	const source = await readFile(filePath, 'utf8');
	componentLineCounts.push({
		relativePath: path.relative(process.cwd(), filePath),
		lineCount: countLines(source)
	});
}

const filesAtOrAboveWarningLimit = componentLineCounts
	.filter(({ lineCount }) => lineCount >= warningLimit)
	.sort((left, right) => right.lineCount - left.lineCount);
const filesAtOrAboveFailureLimit = filesAtOrAboveWarningLimit.filter(
	({ lineCount }) => lineCount >= failureLimit
);
const filesAtOrAboveCriticalLimit = filesAtOrAboveFailureLimit.filter(
	({ lineCount }) => lineCount >= criticalLimit
);

if (filesAtOrAboveWarningLimit.length > 0) {
	console.warn(
		[
			`Vue SRP line-count warnings: ${filesAtOrAboveWarningLimit.length} component(s) are at or above ${warningLimit} lines.`,
			`Longest warning files, capped at ${maxReportedFiles}:`,
			formatFileReport(filesAtOrAboveWarningLimit)
		].join('\n')
	);
}

if (filesAtOrAboveFailureLimit.length > 0) {
	console.error(
		[
			`Vue SRP line-count failures: ${filesAtOrAboveFailureLimit.length} component(s) are at or above ${failureLimit} lines.`,
			`Critical hard-limit failures at or above ${criticalLimit} lines: ${filesAtOrAboveCriticalLimit.length}.`,
			`Longest failing files, capped at ${maxReportedFiles}:`,
			formatFileReport(filesAtOrAboveFailureLimit)
		].join('\n')
	);
	process.exit(1);
}

console.log(
	`Vue SRP line-count check passed: ${vueFiles.length} component(s), no files at or above ${failureLimit} lines.`
);
