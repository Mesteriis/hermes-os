import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';

const sourceRoots = ['src', 'public'];
const checkedExtensions = new Set([
	'.html',
	'.js',
	'.jsx',
	'.mjs',
	'.ts',
	'.tsx',
	'.vue'
]);
const forbiddenStylePatterns = [
	{
		pattern: /(?:^|[\s<])(?<binding>:style|v-bind:style|style)\s*=/,
		message: 'inline style attributes are forbidden; move styles to CSS files or an approved runtime layout primitive'
	}
];

const dynamicLayoutStyleAllowlist = new Set([
	'src/domains/communications/components/AttachmentSearchPanel.vue',
	'src/domains/communications/components/MailList.vue',
	'src/domains/documents/components/DocumentsList.vue',
	'src/domains/knowledge/components/KnowledgeGraphCanvas.vue',
	'src/domains/notes/components/NotesList.vue',
	'src/domains/personas/components/PersonsList.vue',
	'src/domains/tasks/components/TaskList.vue',
	'src/domains/telegram/components/TelegramChatList.vue',
	'src/domains/timeline/components/TimelineStream.vue',
	'src/domains/whatsapp/components/WhatsAppSessionList.vue'
]);

async function collectSourceFiles(root) {
	if (!(await exists(root))) {
		return [];
	}

	const entries = await readdir(root, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const entryPath = path.join(root, entry.name);

		if (entry.isDirectory()) {
			files.push(...(await collectSourceFiles(entryPath)));
			continue;
		}

		if (entry.isFile() && checkedExtensions.has(path.extname(entry.name))) {
			if (isTestFile(entryPath)) {
				continue;
			}
			files.push(entryPath);
		}
	}

	return files;
}

function isTestFile(filePath) {
	const normalized = filePath.split(path.sep).join('/');
	return normalized.includes('/__tests__/') || /\.(test|spec)\.[cm]?[jt]sx?$/.test(normalized);
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

async function findStyleContractViolations() {
	const files = (await Promise.all(sourceRoots.map(collectSourceFiles))).flat();
	const violations = [];

	for (const file of files) {
		const source = await readFile(file, 'utf8');
		const scanLines = scanLinesForFile(file, source);
		const normalizedFile = file.split(path.sep).join('/');

		for (const { line, lineNumber } of scanLines) {
			for (const { pattern, message } of forbiddenStylePatterns) {
				const match = pattern.exec(line);
				if (match) {
					const binding = match.groups?.binding;
					const isAllowedDynamicLayout =
						binding !== 'style' && dynamicLayoutStyleAllowlist.has(normalizedFile);
					if (!isAllowedDynamicLayout) {
						violations.push(`${file}:${lineNumber}: ${message}`);
					}
				}
			}
	}
	}

	return violations;
}

function scanLinesForFile(file, source) {
	const lines = source.split('\n');
	if (path.extname(file) !== '.vue') {
		return lines.map((line, index) => ({ line, lineNumber: index + 1 }));
	}

	const ranges = [];
	let inTemplate = false;
	for (const [index, line] of lines.entries()) {
		if (line.includes('<template')) {
			inTemplate = true;
		}
		if (inTemplate) {
			ranges.push({ line, lineNumber: index + 1 });
		}
		if (line.includes('</template>')) {
			inTemplate = false;
		}
	}

	return ranges;
}

const violations = await findStyleContractViolations();

if (violations.length > 0) {
	console.error(violations.join('\n'));
	process.exit(1);
}
