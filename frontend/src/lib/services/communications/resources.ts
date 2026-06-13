import {
	detectMessageLanguage,
	fetchAttachmentDuplicates,
	fetchCertificates,
	fetchExpiringCertificates,
	fetchInvoices,
	fetchLegalDocuments,
	fetchMailBlockers,
	fetchMessageAuth,
	fetchMessageExplain,
	fetchMessageSignature,
	fetchMessageSmartCc,
	fetchPersonas,
	fetchRichTemplates,
	fetchSubscriptions,
	type MailMessageInsight,
	type MailResourceSnapshot,
	type MailResourceSummary
} from '$lib/api';

export async function loadMessageInsights(messageId: string): Promise<MailMessageInsight> {
	const [explain, smartCc, auth, signature, language] = await Promise.all([
		safe(() => fetchMessageExplain(messageId), null),
		safe(() => fetchMessageSmartCc(messageId), null),
		safe(() => fetchMessageAuth(messageId), null),
		safe(() => fetchMessageSignature(messageId), null),
		safe(() => detectMessageLanguage(messageId), null)
	]);
	return {
		messageId,
		explain,
		smartCc,
		auth,
		signature,
		language,
		aiReply: null,
		tasks: [],
		notes: [],
		translation: null
	};
}

export async function loadMailResources(accountId?: string): Promise<{
	resources: MailResourceSnapshot;
	summary: MailResourceSummary;
}> {
	const [
		subscriptions,
		duplicates,
		invoices,
		legalDocuments,
		certificates,
		expiringCertificates,
		personas,
		templates,
		blockers
	] = await Promise.all([
		safe(() => fetchSubscriptions(accountId), []),
		safe(() => fetchAttachmentDuplicates(), []),
		safe(async () => (await fetchInvoices()).items, []),
		safe(async () => (await fetchLegalDocuments()).items, []),
		safe(async () => (await fetchCertificates()).items, []),
		safe(async () => (await fetchExpiringCertificates()).items, []),
		safe(async () => (await fetchPersonas()).items, []),
		safe(async () => (await fetchRichTemplates()).templates ?? [], []),
		safe(() => fetchMailBlockers(), [])
	]);
	const resources: MailResourceSnapshot = {
		subscriptions,
		duplicates,
		invoices,
		legalDocuments,
		certificates,
		expiringCertificates,
		personas,
		templates,
		blockers
	};
	return { resources, summary: summarizeMailResourceSnapshot(resources) };
}

export function summarizeMailResourceSnapshot(resources: MailResourceSnapshot): MailResourceSummary {
	return {
		subscriptions: resources.subscriptions.length,
		duplicates: resources.duplicates.length,
		invoices: resources.invoices.length,
		legalDocuments: resources.legalDocuments.length,
		certificates: resources.certificates.length,
		expiringCertificates: resources.expiringCertificates.length,
		personas: resources.personas.length,
		templates: resources.templates.length,
		blockers: resources.blockers.length
	};
}

async function safe<T>(task: () => Promise<T>, fallback: T): Promise<T> {
	try {
		return await task();
	} catch {
		return fallback;
	}
}
