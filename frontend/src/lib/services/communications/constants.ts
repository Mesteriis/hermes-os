import type { MailResourceSnapshot } from '$lib/api';

export const COMMUNICATIONS_NAVIGATOR_LIMIT = 5000;

export const emptyMailResourceSnapshot: MailResourceSnapshot = {
	subscriptions: [],
	duplicates: [],
	invoices: [],
	legalDocuments: [],
	certificates: [],
	expiringCertificates: [],
	personas: [],
	templates: [],
	blockers: []
};
