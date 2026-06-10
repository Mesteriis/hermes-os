export function formatDateTime(value: string | null) {
	if (!value) {
		return '';
	}
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		return '';
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date);
}

export function formatBytes(sizeBytes: number) {
	if (sizeBytes < 1024) {
		return `${sizeBytes} B`;
	}
	if (sizeBytes < 1024 * 1024) {
		return `${(sizeBytes / 1024).toFixed(1)} KB`;
	}
	return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatDuration(durationMs: number | null | undefined) {
	if (durationMs == null) {
		return 'n/a';
	}
	if (durationMs < 1000) {
		return `${durationMs} ms`;
	}
	return `${(durationMs / 1000).toFixed(1)} s`;
}

import type { GraphNodeKind, GraphRelationshipType } from '$lib/api';

export function formatNumber(value: number) {
	return new Intl.NumberFormat('en-US').format(value);
}

export function formatGraphKind(kind: GraphNodeKind | string) {
	return kind
		.split('_')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function formatGraphPropertyValue(value: unknown): string {
	if (value === null || value === undefined) {
		return '';
	}
	if (Array.isArray(value)) {
		return value.map(formatGraphPropertyValue).filter(Boolean).join(', ');
	}
	if (typeof value === 'object') {
		return JSON.stringify(value);
	}
	return String(value);
}

export function formatGraphRelationship(type: GraphRelationshipType | string) {
	return type
		.split('_')
		.filter((part) => !['person', 'email', 'address', 'message'].includes(part))
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

export function formatGraphTimestamp(value: string | null) {
	if (!value) {
		return 'No projection yet';
	}
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) {
		return 'Invalid timestamp';
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date);
}

export function formatProjectDate(value: string | null) {
	if (!value) {
		return 'Not set';
	}
	const date = new Date(`${value}T00:00:00`);
	if (Number.isNaN(date.getTime())) {
		return 'Invalid date';
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		year: 'numeric'
	}).format(date);
}

export function formatProjectDateTime(value: string | null) {
	const formatted = formatDateTime(value);
	return formatted || 'No activity';
}
