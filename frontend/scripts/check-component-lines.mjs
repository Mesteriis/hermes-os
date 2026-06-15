import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const checkedExtensions = new Set(['.ts', '.tsx', '.vue']);

const sourceRoot = path.resolve('src');
const warningLimit = 500;
const failureLimit = 700;
const criticalLimit = 1000;
const maxReportedFiles = 20;

export function classifyLineCount(lineCount) {
	return {
		warning: lineCount >= warningLimit,
		failure: lineCount >= failureLimit,
		critical: lineCount >= criticalLimit
	};
}

export function isProductionSourceFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	if (!checkedExtensions.has(path.extname(normalized))) {
		return false;
	}
	if (normalized.includes('/__tests__/')) {
		return false;
	}
	return !/\.(test|spec)\.[cm]?[jt]sx?$/.test(normalized);
}

export function isLineCountCheckedSourceFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	return checkedExtensions.has(path.extname(normalized));
}

async function collectSourceFiles(directory) {
	if (!(await exists(directory))) {
		return [];
	}

	const entries = await readdir(directory, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(directory, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectSourceFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && isLineCountCheckedSourceFile(entryPath)) {
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

async function exists(root) {
	try {
		await stat(root);
		return true;
	} catch (error) {
		if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
			return false;
		}
		throw error;
	}
}

export async function collectLineCountReport(root = sourceRoot) {
	const files = await collectSourceFiles(root);
	const fileReports = [];

	for (const filePath of files) {
		const source = await readFile(filePath, 'utf8');
		fileReports.push({
			relativePath: path.relative(process.cwd(), filePath),
			lineCount: countLines(source)
		});
	}

	const warningFiles = [];
	const failureFiles = [];
	const criticalFiles = [];

	for (const file of fileReports) {
		const classification = classifyLineCount(file.lineCount);
		if (classification.warning) warningFiles.push(file);
		if (classification.failure) failureFiles.push(file);
		if (classification.critical) criticalFiles.push(file);
	}

	const byLineCountDescending = (left, right) => right.lineCount - left.lineCount;

	return {
		totalFiles: fileReports.length,
		warningFiles: warningFiles.sort(byLineCountDescending),
		failureFiles: failureFiles.sort(byLineCountDescending),
		criticalFiles: criticalFiles.sort(byLineCountDescending)
	};
}

async function main() {
	const report = await collectLineCountReport();

	if (report.warningFiles.length > 0) {
		console.warn(
			[
				`Source SRP line-count warnings: ${report.warningFiles.length} source file(s) are at or above ${warningLimit} lines.`,
				`Longest warning files, capped at ${maxReportedFiles}:`,
				formatFileReport(report.warningFiles)
			].join('\n')
		);
	}

	if (report.failureFiles.length > 0) {
		console.error(
			[
				`Source SRP line-count failures: ${report.failureFiles.length} source file(s) are at or above ${failureLimit} lines.`,
				`Critical architecture failures at or above ${criticalLimit} lines: ${report.criticalFiles.length}.`,
				`Longest failing files, capped at ${maxReportedFiles}:`,
				formatFileReport(report.failureFiles)
			].join('\n')
		);
		process.exit(1);
	}

	console.log(
		`Source SRP line-count check passed: ${report.totalFiles} source file(s), no files at or above ${failureLimit} lines.`
	);
}

const entryPointUrl = process.argv[1] ? pathToFileURL(process.argv[1]).href : '';

if (import.meta.url === entryPointUrl) {
	await main();
}
