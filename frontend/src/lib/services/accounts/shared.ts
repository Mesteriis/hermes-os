export function safeAccountIdSegment(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9_-]+/g, '-')
		.replace(/^-+|-+$/g, '')
		.slice(0, 48);
}

export function optionalTrimmed(value: string): string | undefined {
	const trimmed = value.trim();
	return trimmed ? trimmed : undefined;
}
