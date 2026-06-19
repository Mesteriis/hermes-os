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
const forbiddenCanonicalEvidenceDirs = [
	'backend/src/domains/signals',
	'backend/src/domains/events',
	'backend/src/domains/attention',
	'backend/src/domains/evidence',
	'backend/src/vault/observations'
];
const communicationRawRecordInsertOwner = 'backend/src/domains/mail/core/raw_records.rs';
const communicationMessageInsertOwner = 'backend/src/domains/mail/messages/store/upsert.rs';
const reviewPromotionEngine = 'backend/src/engines/review_promotion/mod.rs';
const communicationProviderCrudFacadeOwners = new Set([
	'backend/src/domains/mail/core/accounts.rs',
	'backend/src/domains/mail/core/secrets.rs'
]);
const telegramProviderOwnershipCompatExceptions = new Set([
	'backend/src/integrations/telegram/api/raw.rs'
]);
const telegramCommandQueueOwner = 'backend/src/integrations/telegram/client/commands.rs';
const mailSyncRunMutationOwners = new Set([
	'backend/src/domains/mail/background_sync/store/run_start.rs',
	'backend/src/domains/mail/background_sync/store/run_progress.rs',
	'backend/src/domains/mail/background_sync/store/run_finish.rs',
	'backend/src/domains/mail/background_sync/store/orphaned.rs'
]);
const aiPromptMutationOwners = new Set([
	'backend/src/ai/control_center/prompts/templates.rs',
	'backend/src/ai/control_center/prompts/versions.rs',
	'backend/src/ai/control_center/prompts/activation.rs',
	'backend/src/ai/control_center/prompts/evaluation.rs'
]);
const aiModelCatalogMutationOwners = new Set([
	'backend/src/ai/control_center/catalog.rs'
]);
const aiModelRouteMutationOwners = new Set([
	'backend/src/ai/control_center/routes.rs'
]);
const aiSemanticEmbeddingMutationOwners = new Set([
	'backend/src/ai/core/semantic/embeddings.rs'
]);
const documentProcessingJobMutationOwners = new Set([
	'backend/src/domains/documents/processing/jobs.rs'
]);
const whatsappSessionMutationOwners = new Set([
	'backend/src/integrations/whatsapp/client/store/sessions.rs'
]);
const telegramChatMutationOwners = new Set([
	'backend/src/integrations/telegram/client/chats.rs'
]);
const telegramChatMutationCompatExceptions = new Set([
	'backend/src/integrations/telegram/client/participant_roster.rs'
]);
const telegramChatParticipantMutationOwners = new Set([
	'backend/src/integrations/telegram/client/participants.rs'
]);
const telegramChatParticipantMutationCompatExceptions = new Set([
	'backend/src/integrations/telegram/client/participant_roster.rs'
]);
const telegramTopicMutationOwners = new Set([
	'backend/src/integrations/telegram/client/topics.rs'
]);
const telegramReactionMutationOwners = new Set([
	'backend/src/integrations/telegram/client/reactions.rs'
]);
const automationTemplatePolicyMutationOwners = new Set([
	'backend/src/engines/automation/store.rs'
]);
const automationOutboundMessageMutationOwners = new Set([
	'backend/src/engines/automation/dry_run.rs'
]);
const telegramMessageVersionMutationOwners = new Set([
	'backend/src/integrations/telegram/client/lifecycle/message_versions.rs'
]);
const telegramMessageTombstoneMutationOwners = new Set([
	'backend/src/integrations/telegram/client/lifecycle/tombstones.rs'
]);
const telegramCommunicationMessageUpdateOwners = new Set([
	'backend/src/integrations/telegram/client/messages/provider_state.rs',
	'backend/src/integrations/telegram/client/messages/attachments.rs'
]);
const reviewManualOrchestrationOwner = 'backend/src/domains/review/service.rs';
const taskCommandServiceOwner = 'backend/src/domains/tasks/service.rs';
const calendarCommandServiceOwner = 'backend/src/domains/calendar/service.rs';
const organizationCommandServiceOwner = 'backend/src/domains/organizations/service.rs';
const personCommandServiceOwner = 'backend/src/domains/persons/service.rs';
const decisionCommandServiceOwner = 'backend/src/domains/decisions/service.rs';
const obligationCommandServiceOwner = 'backend/src/domains/obligations/service.rs';
const relationshipCommandServiceOwner = 'backend/src/domains/relationships/service.rs';
const taskCandidateReviewServiceOwner = 'backend/src/domains/tasks/candidates/service.rs';
const projectLinkReviewServiceOwner = 'backend/src/domains/projects/link_reviews/service.rs';
const contradictionReviewServiceOwner = 'backend/src/engines/consistency/service.rs';
const documentProcessingCommandServiceOwner = 'backend/src/domains/documents/processing/service.rs';
const mailCommandServiceOwner = 'backend/src/domains/mail/service.rs';
const emailSyncPipelineOrganizationOwner = 'backend/src/workflows/email_sync_pipeline/organizations.rs';
const emailSyncPipelineParticipantsOwner = 'backend/src/workflows/email_sync_pipeline/participants.rs';
const emailSyncPipelineRelationshipsOwner = 'backend/src/workflows/email_sync_pipeline/relationships.rs';

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

function canonicalEvidenceBoundaryFailures(trackedFiles, existingDirs = new Set()) {
	const errors = [];
	const rejectedFiles = new Set();

	for (const forbiddenDir of forbiddenCanonicalEvidenceDirs) {
		if (
			existingDirs.has(forbiddenDir) ||
			trackedFiles.some((file) => file === forbiddenDir || file.startsWith(`${forbiddenDir}/`))
		) {
			errors.push(`${forbiddenDir}: forbidden boundary; observations belong to platform/observations, not Vault or a domain`);
			for (const file of trackedFiles) {
				if (file === forbiddenDir || file.startsWith(`${forbiddenDir}/`)) {
					rejectedFiles.add(file);
				}
			}
		}
	}

	for (const file of trackedFiles) {
		if (!rejectedFiles.has(file) && /^backend\/src\/vault\/.*observations?/i.test(file)) {
			errors.push(`${file}: Vault must not own observations; use backend/src/platform/observations`);
		}
	}

	return errors;
}

function canonicalCommunicationRawRecordWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		if (file === communicationRawRecordInsertOwner) continue;
		if (/\bINSERT\s+INTO\s+communication_raw_records\b/i.test(content)) {
			errors.push(`${file}: communication_raw_records writes must go through ${communicationRawRecordInsertOwner} so every raw provider record has a canonical observation`);
		}
	}
	return errors;
}

function canonicalCommunicationMessageWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		if (file === communicationMessageInsertOwner) continue;
		if (/\bINSERT\s+INTO\s+communication_messages\b/i.test(content)) {
			errors.push(`${file}: communication_messages writes must go through ${communicationMessageInsertOwner} so every message projection keeps its canonical observation reference`);
		}
	}
	return errors;
}

function canonicalTaskCandidateWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+task_candidates\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: task_candidates writes must include observation_id so message candidates stay linked to canonical evidence`);
			}
		}
	}
	return errors;
}

function canonicalGraphEvidenceWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+graph_evidence\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: graph_evidence writes must include observation_id so knowledge graph evidence can point at canonical observations`);
			}
		}
	}
	return errors;
}

function canonicalSemanticEmbeddingWriteFailures(fileContents) {
	const errors = [];
	for (const [file, content] of fileContents.entries()) {
		const insertPattern = /\bINSERT\s+INTO\s+semantic_embeddings\s*\(([\s\S]*?)\)\s*VALUES\b/gi;
		for (const match of content.matchAll(insertPattern)) {
			if (!/\bobservation_id\b/i.test(match[1])) {
				errors.push(`${file}: semantic_embeddings writes must include observation_id so AI retrieval can cite canonical evidence`);
			}
		}
	}
	return errors;
}

function canonicalReviewPromotionEvidenceFailures(fileContents) {
	const content = fileContents.get(reviewPromotionEngine);
	if (content === undefined) return [];
	if (/\b[A-Za-z]*EvidenceSourceKind::RawRecord\b/.test(content)) {
		return [
			`${reviewPromotionEngine}: review promotion evidence must use observation source kind and observation_id, not raw_record compatibility evidence`
		];
	}
	return [];
}

function canonicalReviewPromotionOwnerFailures(fileContents) {
	const content = fileContents.get(reviewPromotionEngine);
	if (content === undefined) return [];
	if (/\bINSERT\s+INTO\s+(persons|person_personas|organizations|projects|project_keywords|obligation_task_links)\b/i.test(content)) {
		return [
			`${reviewPromotionEngine}: review promotion must materialize persons/organizations/projects/task-links through their domain stores, not direct SQL owners`
		];
	}
	return [];
}

function emailSyncPipelineOrganizationOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineOrganizationOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+(organizations|organization_domains|organization_identities|organization_contact_links)\b/i.test(content)) {
		return [
			`${emailSyncPipelineOrganizationOwner}: email sync organization projection must use organization domain owner stores, not direct SQL writes`
		];
	}
	return [];
}

function emailSyncPipelineParticipantsOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineParticipantsOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+communication_message_participants\b/i.test(content)) {
		return [
			`${emailSyncPipelineParticipantsOwner}: email sync message participant projection must use message domain owner stores, not direct SQL writes`
		];
	}
	return [];
}

function emailSyncPipelineRelationshipsOwnerFailures(fileContents) {
	const content = fileContents.get(emailSyncPipelineRelationshipsOwner);
	if (content === undefined) return [];
	if (/\b(?:INSERT\s+INTO|UPDATE)\s+relationship_events\b/i.test(content)) {
		return [
			`${emailSyncPipelineRelationshipsOwner}: email sync relationship timeline projection must use relationship event owner stores, not direct SQL writes`
		];
	}
	return [];
}

function legacyContextPackTableFailures(fileContents) {
	const errors = [];
	const legacyContextPackPattern = /\b(event_context_packs|task_context_packs)\b/;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/')) continue;
		if (legacyContextPackPattern.test(content)) {
			errors.push(
				`${file}: legacy event_context_packs/task_context_packs access is forbidden; use backend/src/engines/context_packs through domain compatibility stores`
			);
		}
	}
	return errors;
}

function automationTemplatePolicyMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+(automation_templates|automation_policies)\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/engines/automation')) continue;
		if (automationTemplatePolicyMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: automation template/policy mutations must stay in backend/src/engines/automation/store.rs so policy configuration writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function automationOutboundMessageMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+telegram_outbound_messages\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/engines/automation')) continue;
		if (automationOutboundMessageMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_outbound_messages mutations for automation dry-run/live send must stay in backend/src/engines/automation/dry_run.rs so outbound materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function communicationProviderCrudFacadeFailures(fileContents) {
	const errors = [];
	const forbiddenUsagePattern =
		/\.(?:upsert_provider_account|provider_account|list_provider_accounts|bind_provider_account_secret|provider_account_secret_bindings|provider_account_secret_binding)\s*\(/g;
	for (const [file, content] of fileContents.entries()) {
		if (communicationProviderCrudFacadeOwners.has(file)) continue;
		if (forbiddenUsagePattern.test(content)) {
			errors.push(
				`${file}: provider account and secret binding ownership must use crate::vault::*Store directly, not CommunicationIngestionStore compatibility CRUD`
			);
		}
	}
	return errors;
}

function providerAccountOwnerMutationFailures(fileContents) {
	const errors = [];
	const ownerFile = 'backend/src/vault/provider_accounts.rs';
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE|DELETE\s+FROM)\s+(?:communication_provider_accounts|communication_provider_account_secret_refs|task_provider_accounts|calendar_accounts|calendar_sources)\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/')) continue;
		if (file === ownerFile) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: provider account/source durable mutations must stay in ${ownerFile} so vault-owned records keep canonical evidence ownership`
			);
		}
	}
	return errors;
}

function telegramProviderOwnershipFailures(fileContents) {
	const errors = [];
	const forbiddenUsagePattern =
		/\bCommunicationIngestionStore\b|\bcommunication_ingestion_store\s*\(/g;
	for (const [file, content] of fileContents.entries()) {
		const isTelegramRuntime = file.startsWith('backend/src/integrations/telegram/runtime/');
		const isTelegramApi = file.startsWith('backend/src/integrations/telegram/api/');
		if (!isTelegramRuntime && !isTelegramApi) continue;
		if (telegramProviderOwnershipCompatExceptions.has(file)) continue;
		if (forbiddenUsagePattern.test(content)) {
			errors.push(
				`${file}: Telegram runtime/API provider ownership must use crate::vault::CommunicationProvider*Store, not CommunicationIngestionStore`
			);
		}
	}
	return errors;
}

function telegramCommandQueueMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT|UPDATE)\s+telegram_provider_write_commands\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (file === telegramCommandQueueOwner) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram provider command queue mutations must go through ${telegramCommandQueueOwner} so command lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function mailSyncRunMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+communication_mail_sync_runs\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/domains/mail/background_sync/')) continue;
		if (mailSyncRunMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: communication_mail_sync_runs mutations must stay in the dedicated owner store files so run lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiPromptMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern =
		/\b(?:INSERT\s+INTO|UPDATE)\s+(ai_prompt_templates|ai_prompt_template_versions|ai_prompt_eval_runs)\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiPromptMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: AI prompt template/version/eval mutations must stay in dedicated prompt owner files so prompt studio writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiModelCatalogMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+ai_model_catalog\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiModelCatalogMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: ai_model_catalog mutations must stay in backend/src/ai/control_center/catalog.rs so curated model materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function aiModelRouteMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+ai_model_routes\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/control_center/')) continue;
		if (aiModelRouteMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: ai_model_routes mutations must stay in backend/src/ai/control_center/routes.rs so model route writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function aiSemanticEmbeddingMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+semantic_embeddings\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/ai/core/semantic/')) continue;
		if (aiSemanticEmbeddingMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: semantic_embeddings mutations must stay in backend/src/ai/core/semantic/embeddings.rs so derived embedding materialization always emits canonical observation trail`
			);
		}
	}
	return errors;
}

function documentProcessingJobMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+document_processing_jobs\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/domains/documents/processing/')) continue;
		if (documentProcessingJobMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: document_processing_jobs mutations must stay in backend/src/domains/documents/processing/jobs.rs so job lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function whatsappSessionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+whatsapp_web_sessions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/whatsapp/')) continue;
		if (whatsappSessionMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: whatsapp_web_sessions mutations must stay in backend/src/integrations/whatsapp/client/store/sessions.rs so session lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramChatMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_chats\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramChatMutationOwners.has(file)) continue;
		if (telegramChatMutationCompatExceptions.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_chats mutations must stay in backend/src/integrations/telegram/client/chats.rs so chat lifecycle writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramChatParticipantMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_chat_participants\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramChatParticipantMutationOwners.has(file)) continue;
		if (telegramChatParticipantMutationCompatExceptions.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_chat_participants mutations must stay in backend/src/integrations/telegram/client/participants.rs so roster writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramTopicMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_topics\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramTopicMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_topics mutations must stay in backend/src/integrations/telegram/client/topics.rs so topic projection writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramReactionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\b(?:INSERT\s+INTO|UPDATE)\s+telegram_message_reactions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramReactionMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_reactions mutations must stay in backend/src/integrations/telegram/client/reactions.rs so reaction writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramMessageVersionMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\bINSERT\s+INTO\s+telegram_message_versions\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramMessageVersionMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_versions mutations must stay in backend/src/integrations/telegram/client/lifecycle/message_versions.rs so edit-version writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramMessageTombstoneMutationFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\bINSERT\s+INTO\s+telegram_message_tombstones\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramMessageTombstoneMutationOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram_message_tombstones mutations must stay in backend/src/integrations/telegram/client/lifecycle/tombstones.rs so tombstone writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function telegramCommunicationMessageUpdateFailures(fileContents) {
	const errors = [];
	const directMutationPattern = /\bUPDATE\s+communication_messages\b/gi;
	for (const [file, content] of fileContents.entries()) {
		if (!file.startsWith('backend/src/integrations/telegram/')) continue;
		if (telegramCommunicationMessageUpdateOwners.has(file)) continue;
		if (directMutationPattern.test(content)) {
			errors.push(
				`${file}: telegram communication_messages projection updates must stay in backend/src/integrations/telegram/client/messages/{provider_state,attachments}.rs so shared message projection writes always emit canonical observation trail`
			);
		}
	}
	return errors;
}

function reviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/review/api.rs');
	if (content === undefined) return [];

	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bReviewPromotionService\b/,
		/\.promote_with_observation\s*\(/,
		/\.set_status_with_observation\s*\(/
	];

	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/review/api.rs: manual review transition/promotion orchestration must stay in ${reviewManualOrchestrationOwner}, not the API layer`
		];
	}

	return [];
}

function tasksHandlerManualOrchestrationFailures(fileContents) {
	const taskHandlerFiles = [
		'backend/src/domains/tasks/handlers/tasks.rs',
		'backend/src/domains/tasks/handlers/core_records.rs',
		'backend/src/domains/tasks/handlers/intelligence.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bTaskStore::new\s*\([^)]+\)\.create\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.set_status_with_observation\s*\(/,
		/\.archive_with_observation\s*\(/,
		/\.add_with_source\s*\(/
	];

	const errors = [];
	for (const file of taskHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual task mutation orchestration must stay in ${taskCommandServiceOwner}, not task handlers`
			);
		}
	}
	return errors;
}

function calendarHandlerManualOrchestrationFailures(fileContents) {
	const calendarHandlerFiles = [
		'backend/src/domains/calendar/handlers/accounts.rs',
		'backend/src/domains/calendar/handlers/events/agenda.rs',
		'backend/src/domains/calendar/handlers/events/checklist.rs',
		'backend/src/domains/calendar/handlers/events/participants.rs',
		'backend/src/domains/calendar/handlers/events/relations.rs',
		'backend/src/domains/calendar/handlers/meetings.rs',
		'backend/src/domains/calendar/handlers/reminders.rs',
		'backend/src/domains/calendar/handlers/rules.rs',
		'backend/src/domains/calendar/handlers/scheduling.rs',
		'backend/src/domains/calendar/handlers/sync.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.set_with_observation\s*\(/,
		/\.add_with_observation\s*\(/,
		/\.link_with_observation\s*\(/,
		/\.set_active_with_observation\s*\(/
	];

	const errors = [];
	for (const file of calendarHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual calendar mutation orchestration must stay in ${calendarCommandServiceOwner}, not calendar handlers`
			);
		}
	}
	return errors;
}

function organizationHandlerManualOrchestrationFailures(fileContents) {
	const organizationHandlerFiles = [
		'backend/src/domains/organizations/handlers/organizations.rs',
		'backend/src/domains/organizations/handlers/core_records.rs',
		'backend/src/domains/organizations/handlers/enrichment.rs',
		'backend/src/domains/organizations/handlers/health.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.archive_with_observation\s*\(/,
		/\.upsert_with_observation\s*\(/,
		/\.add_with_observation\s*\(/,
		/\.link_with_observation\s*\(/,
		/\.apply_with_observation\s*\(/,
		/\.toggle_watchlist_with_observation\s*\(/
	];

	const errors = [];
	for (const file of organizationHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual organization mutation orchestration must stay in ${organizationCommandServiceOwner}, not organization handlers`
			);
		}
	}
	return errors;
}

function personHandlerManualOrchestrationFailures(fileContents) {
	const personHandlerFiles = [
		'backend/src/domains/persons/handlers/compatibility.rs',
		'backend/src/domains/persons/handlers/health.rs',
		'backend/src/domains/persons/handlers/history.rs',
		'backend/src/domains/persons/handlers/identity.rs',
		'backend/src/domains/persons/handlers/intelligence.rs',
		'backend/src/domains/persons/handlers/investigator.rs',
		'backend/src/domains/persons/handlers/memory.rs',
		'backend/src/domains/persons/handlers/profile/actions.rs',
		'backend/src/domains/persons/handlers/profile/owner.rs',
		'backend/src/domains/persons/handlers/profile/personas.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.create_unattached_with_observation\s*\(/,
		/\.attach_to_persona_with_observation\s*\(/,
		/\.upsert_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.assign_with_observation\s*\(/,
		/\.remove_with_observation\s*\(/,
		/\.apply_with_observation\s*\(/,
		/\.reject_with_observation\s*\(/,
		/\.toggle_watchlist_with_observation\s*\(/,
		/\.enrich_person_with_observation\s*\(/,
		/\.toggle_favorite_with_observation\s*\(/,
		/\.set_notes_with_observation\s*\(/,
		/\.set_owner_persona_with_observation\s*\(/,
		/\.update_persona_with_observation\s*\(/,
		/\.set_review_state_with_observation\s*\(/,
		/\.review_dossier_snapshot_with_observation\s*\(/,
		/\.add_with_observation\s*\(/
	];

	const errors = [];
	for (const file of personHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual person mutation/review orchestration must stay in ${personCommandServiceOwner}, not person handlers`
			);
		}
	}
	return errors;
}

function mailCommunicationQueryManualOrchestrationFailures(fileContents) {
	const mailHandlerFiles = [
		'backend/src/domains/mail/handlers/communication_queries/drafts.rs',
		'backend/src/domains/mail/handlers/communication_queries/folders.rs',
		'backend/src/domains/mail/handlers/communication_queries/saved_searches.rs',
		'backend/src/domains/mail/handlers/communication_queries/outbox.rs',
		'backend/src/domains/mail/handlers/communication_queries/imports.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.upsert_with_observation\s*\(/,
		/\.delete_with_observation\s*\(/,
		/\.create_with_observation\s*\(/,
		/\.update_with_observation\s*\(/,
		/\.copy_message_with_observation\s*\(/,
		/\.move_message_with_observation\s*\(/,
		/\.undo_with_observation\s*\(/,
		/\.upsert_imported_attachment_with_observation\s*\(/
	];

	const errors = [];
	for (const file of mailHandlerFiles) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: manual mail communication mutation orchestration must stay in ${mailCommandServiceOwner}, not communication query handlers`
			);
		}
	}
	return errors;
}

function mailProviderSendManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/mail/handlers/sending/provider_send.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.record_sent_with_observation\s*\(/,
		/\.enqueue_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/mail/handlers/sending/provider_send.rs: manual provider send evidence orchestration must stay in ${mailCommandServiceOwner}, not the sending handler`
		];
	}
	return [];
}

function mailFinalHandlerManualOrchestrationFailures(fileContents) {
	const files = [
		'backend/src/domains/mail/handlers/sending/forwarding.rs',
		'backend/src/domains/mail/handlers/workflow_state.rs',
		'backend/src/domains/mail/handlers/sending/local_state.rs',
		'backend/src/domains/mail/handlers/message_ai_state.rs',
		'backend/src/domains/mail/handlers/message_actions.rs',
		'backend/src/domains/mail/handlers/workflow_actions/actions/persons.rs'
	];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\bObservationStore::capture_in_transaction\b/,
		/\.transition_workflow_state_with_observation\s*\(/,
		/\.move_to_local_trash_with_observation\s*\(/,
		/\.restore_from_local_trash_with_observation\s*\(/,
		/\.transition_with_observation\s*\(/,
		/\.toggle_pin_with_observation\s*\(/,
		/\.toggle_important_with_observation\s*\(/,
		/\.snooze_with_observation\s*\(/,
		/\.toggle_mute_with_observation\s*\(/,
		/\.add_label_with_observation\s*\(/,
		/\.remove_label_with_observation\s*\(/,
		/\.enqueue_with_observation\s*\(/,
		/\.link_email_person_projection_in_transaction\s*\(/
	];

	const errors = [];
	for (const file of files) {
		const content = fileContents.get(file);
		if (content === undefined) continue;
		if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
			errors.push(
				`${file}: final mail handler observation/projection orchestration must stay in ${mailCommandServiceOwner}, not the handler layer`
			);
		}
	}
	return errors;
}

function mailAccountManagementManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/mail/handlers/account_management.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bObservationOriginKind\b/,
		/\.update_config_with_origin\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/mail/handlers/account_management.rs: email account logout/config mutation orchestration must stay in backend/src/vault/provider_accounts.rs owner methods, not the handler`
		];
	}
	return [];
}

function decisionHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/decisions/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/decisions/api/handlers.rs: manual decision review orchestration must stay in ${decisionCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function obligationHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/obligations/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/obligations/api/handlers.rs: manual obligation review orchestration must stay in ${obligationCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function relationshipHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/relationships/api/handlers.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/relationships/api/handlers.rs: manual relationship review orchestration must stay in ${relationshipCommandServiceOwner}, not the API layer`
		];
	}
	return [];
}

function taskCandidateHandlerManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/tasks/handlers/candidates.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\.capture\s*\(/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/tasks/handlers/candidates.rs: manual task candidate review orchestration must stay in ${taskCandidateReviewServiceOwner}, not the handler`
		];
	}
	return [];
}

function projectLinkReviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/projects/api/mod.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/projects/api/mod.rs: manual project link review orchestration must stay in ${projectLinkReviewServiceOwner}, not the API layer`
		];
	}
	return [];
}

function contradictionReviewApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/engines/consistency_api.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore\b/,
		/\.set_review_state_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/engines/consistency_api.rs: manual contradiction review orchestration must stay in ${contradictionReviewServiceOwner}, not the API layer`
		];
	}
	return [];
}

function documentProcessingApiManualOrchestrationFailures(fileContents) {
	const content = fileContents.get('backend/src/domains/documents/api/mod.rs');
	if (content === undefined) return [];
	const forbiddenPatterns = [
		/\bNewObservation\b/,
		/\bObservationOriginKind\b/,
		/\bObservationStore::new\b/,
		/\.retry_failed_job_with_observation\s*\(/
	];
	if (forbiddenPatterns.some((pattern) => pattern.test(content))) {
		return [
			`backend/src/domains/documents/api/mod.rs: manual document-processing retry orchestration must stay in ${documentProcessingCommandServiceOwner}, not the API layer`
		];
	}
	return [];
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

async function checkCanonicalEvidenceBoundaries() {
	const trackedFiles = await gitLsFiles();
	const vaultFiles = await collectFiles('backend/src/vault', new Set(['.rs']));
	const files = [...new Set([...trackedFiles, ...vaultFiles])];
	const existingDirs = new Set();
	for (const forbiddenDir of forbiddenCanonicalEvidenceDirs) {
		if (await exists(path.join(repoRoot, forbiddenDir))) {
			existingDirs.add(forbiddenDir);
		}
	}
	failures.push(...canonicalEvidenceBoundaryFailures(files, existingDirs));

	const backendFiles = await collectFiles('backend/src', new Set(['.rs']));
	const fileContents = new Map();
	for (const file of backendFiles) {
		fileContents.set(file, await readFile(path.join(repoRoot, file), 'utf8'));
	}
	failures.push(...canonicalCommunicationRawRecordWriteFailures(fileContents));
	failures.push(...canonicalCommunicationMessageWriteFailures(fileContents));
	failures.push(...canonicalTaskCandidateWriteFailures(fileContents));
	failures.push(...canonicalGraphEvidenceWriteFailures(fileContents));
	failures.push(...canonicalSemanticEmbeddingWriteFailures(fileContents));
	failures.push(...canonicalReviewPromotionEvidenceFailures(fileContents));
	failures.push(...canonicalReviewPromotionOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineOrganizationOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineParticipantsOwnerFailures(fileContents));
	failures.push(...emailSyncPipelineRelationshipsOwnerFailures(fileContents));
	failures.push(...legacyContextPackTableFailures(fileContents));
	failures.push(...automationTemplatePolicyMutationFailures(fileContents));
	failures.push(...automationOutboundMessageMutationFailures(fileContents));
	failures.push(...communicationProviderCrudFacadeFailures(fileContents));
	failures.push(...telegramProviderOwnershipFailures(fileContents));
	failures.push(...telegramCommandQueueMutationFailures(fileContents));
	failures.push(...mailSyncRunMutationFailures(fileContents));
	failures.push(...aiPromptMutationFailures(fileContents));
	failures.push(...aiModelCatalogMutationFailures(fileContents));
	failures.push(...aiModelRouteMutationFailures(fileContents));
	failures.push(...aiSemanticEmbeddingMutationFailures(fileContents));
	failures.push(...documentProcessingJobMutationFailures(fileContents));
	failures.push(...whatsappSessionMutationFailures(fileContents));
	failures.push(...telegramChatMutationFailures(fileContents));
	failures.push(...telegramChatParticipantMutationFailures(fileContents));
	failures.push(...telegramTopicMutationFailures(fileContents));
	failures.push(...telegramReactionMutationFailures(fileContents));
	failures.push(...telegramMessageVersionMutationFailures(fileContents));
	failures.push(...telegramMessageTombstoneMutationFailures(fileContents));
	failures.push(...telegramCommunicationMessageUpdateFailures(fileContents));
	failures.push(...providerAccountOwnerMutationFailures(fileContents));
	failures.push(...reviewApiManualOrchestrationFailures(fileContents));
	failures.push(...tasksHandlerManualOrchestrationFailures(fileContents));
	failures.push(...calendarHandlerManualOrchestrationFailures(fileContents));
	failures.push(...organizationHandlerManualOrchestrationFailures(fileContents));
	failures.push(...personHandlerManualOrchestrationFailures(fileContents));
	failures.push(...mailCommunicationQueryManualOrchestrationFailures(fileContents));
	failures.push(...mailProviderSendManualOrchestrationFailures(fileContents));
	failures.push(...mailFinalHandlerManualOrchestrationFailures(fileContents));
	failures.push(...mailAccountManagementManualOrchestrationFailures(fileContents));
	failures.push(...decisionHandlerManualOrchestrationFailures(fileContents));
	failures.push(...obligationHandlerManualOrchestrationFailures(fileContents));
	failures.push(...relationshipHandlerManualOrchestrationFailures(fileContents));
	failures.push(...taskCandidateHandlerManualOrchestrationFailures(fileContents));
	failures.push(...projectLinkReviewApiManualOrchestrationFailures(fileContents));
	failures.push(...contradictionReviewApiManualOrchestrationFailures(fileContents));
	failures.push(...documentProcessingApiManualOrchestrationFailures(fileContents));
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
	assertSelfTest(
		'canonical evidence guard rejects forbidden evidence domain',
		canonicalEvidenceBoundaryFailures(['backend/src/domains/evidence/mod.rs']).length === 1
	);
	assertSelfTest(
		'canonical evidence guard rejects vault observations owner',
		canonicalEvidenceBoundaryFailures(['backend/src/vault/observations/mod.rs']).length === 1
	);
	assertSelfTest(
		'canonical evidence guard allows platform observations',
		canonicalEvidenceBoundaryFailures(['backend/src/platform/observations/mod.rs']).length === 0
	);
	assertSelfTest(
		'canonical communication raw record guard rejects direct writes',
		canonicalCommunicationRawRecordWriteFailures(new Map([
			['backend/src/integrations/telegram/client/messages.rs', 'INSERT INTO communication_raw_records (raw_record_id) VALUES ($1)']
		])).length === 1
	);
	assertSelfTest(
		'canonical communication raw record guard allows owner writes',
		canonicalCommunicationRawRecordWriteFailures(new Map([
			[communicationRawRecordInsertOwner, 'INSERT INTO communication_raw_records (raw_record_id) VALUES ($1)']
		])).length === 0
	);
	assertSelfTest(
		'canonical communication message guard rejects direct writes',
		canonicalCommunicationMessageWriteFailures(new Map([
			['backend/src/integrations/slack/messages.rs', 'INSERT INTO communication_messages (message_id) VALUES ($1)']
		])).length === 1
	);
	assertSelfTest(
		'canonical communication message guard allows projection owner writes',
		canonicalCommunicationMessageWriteFailures(new Map([
			[communicationMessageInsertOwner, 'INSERT INTO communication_messages (message_id) VALUES ($1)']
		])).length === 0
	);
	assertSelfTest(
		'canonical task candidate guard rejects writes without observation_id',
		canonicalTaskCandidateWriteFailures(new Map([
			['backend/src/ai/core/service/task_candidate_persistence.rs', 'INSERT INTO task_candidates (task_candidate_id, source_kind, source_id) VALUES ($1, $2, $3)']
		])).length === 1
	);
	assertSelfTest(
		'canonical task candidate guard allows writes with observation_id',
		canonicalTaskCandidateWriteFailures(new Map([
			['backend/src/domains/tasks/candidates/persistence.rs', 'INSERT INTO task_candidates (task_candidate_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4)']
		])).length === 0
	);
	assertSelfTest(
		'canonical graph evidence guard rejects writes without observation_id',
		canonicalGraphEvidenceWriteFailures(new Map([
			['backend/src/domains/graph/core/store.rs', 'INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id) VALUES ($1, $2, $3, $4)']
		])).length === 1
	);
	assertSelfTest(
		'canonical graph evidence guard allows writes with observation_id',
		canonicalGraphEvidenceWriteFailures(new Map([
			['backend/src/domains/graph/core/store.rs', 'INSERT INTO graph_evidence (evidence_id, edge_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4, $5)']
		])).length === 0
	);
	assertSelfTest(
		'canonical semantic embedding guard rejects writes without observation_id',
		canonicalSemanticEmbeddingWriteFailures(new Map([
			['backend/src/ai/core/semantic/embeddings.rs', 'INSERT INTO semantic_embeddings (semantic_embedding_id, source_kind, source_id) VALUES ($1, $2, $3)']
		])).length === 1
	);
	assertSelfTest(
		'canonical semantic embedding guard allows writes with observation_id',
		canonicalSemanticEmbeddingWriteFailures(new Map([
			['backend/src/ai/core/semantic/embeddings.rs', 'INSERT INTO semantic_embeddings (semantic_embedding_id, source_kind, source_id, observation_id) VALUES ($1, $2, $3, $4)']
		])).length === 0
	);
	assertSelfTest(
		'canonical review promotion guard rejects raw_record evidence',
		canonicalReviewPromotionEvidenceFailures(new Map([
			[reviewPromotionEngine, 'NewDecisionEvidence::new(DecisionEvidenceSourceKind::RawRecord, observation_id)']
		])).length === 1
	);
	assertSelfTest(
		'canonical review promotion guard allows observation evidence',
		canonicalReviewPromotionEvidenceFailures(new Map([
			[reviewPromotionEngine, 'NewDecisionEvidence::observation(observation_id)']
		])).length === 0
	);
	assertSelfTest(
		'provider CRUD compatibility facade usage outside owner files fails',
		communicationProviderCrudFacadeFailures(new Map([
			['backend/src/domains/mail/outbox/provider_sender.rs', 'store.upsert_provider_account(&account);']
		])).length === 1
	);
	assertSelfTest(
		'provider account owner mutation guard rejects non-owner durable writes',
		providerAccountOwnerMutationFailures(new Map([
			[
				'backend/src/integrations/telegram/runtime/manager/message_events.rs',
				'INSERT INTO communication_provider_accounts (account_id) VALUES ($1)'
			]
		])).length === 1
	);
	assertSelfTest(
		'provider account owner mutation guard allows vault owner file',
		providerAccountOwnerMutationFailures(new Map([
			[
				'backend/src/vault/provider_accounts.rs',
				'INSERT INTO communication_provider_accounts (account_id) VALUES ($1)'
			]
		])).length === 0
	);
	assertSelfTest(
		'provider CRUD compatibility facade owner files pass',
		communicationProviderCrudFacadeFailures(new Map([
			['backend/src/domains/mail/core/accounts.rs', 'store.upsert_provider_account(&account);']
		])).length === 0
	);
	assertSelfTest(
		'telegram provider ownership guard rejects compatibility store in runtime',
		telegramProviderOwnershipFailures(new Map([
			[
				'backend/src/integrations/telegram/runtime/manager.rs',
				'use crate::domains::mail::core::CommunicationIngestionStore;'
			]
		])).length === 1
	);
	assertSelfTest(
		'telegram provider ownership guard allows raw API exception',
		telegramProviderOwnershipFailures(new Map([
			[
				'backend/src/integrations/telegram/api/raw.rs',
				'let store = communication_ingestion_store(&state)?;'
			]
		])).length === 0
	);
	assertSelfTest(
		'mail account management guard rejects handler-owned logout mutation orchestration',
		mailAccountManagementManualOrchestrationFailures(new Map([
			[
				'backend/src/domains/mail/handlers/account_management.rs',
				'CommunicationProviderAccountStore::new(pool).update_config_with_origin(&account_id, &config, ObservationOriginKind::LocalRuntime, "actor", "logout");'
			]
		])).length === 1
	);
	assertSelfTest(
		'mail account management guard allows owner method call',
		mailAccountManagementManualOrchestrationFailures(new Map([
			[
				'backend/src/domains/mail/handlers/account_management.rs',
				'CommunicationProviderAccountStore::new(pool).mark_logged_out(&account_id);'
			]
		])).length === 0
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
	await checkCanonicalEvidenceBoundaries();
	await checkLayerBoundaries();

	if (failures.length > 0) {
		console.error(failures.join('\n'));
		process.exit(1);
	}

	console.log('Architecture guard passed.');
}

await main();
