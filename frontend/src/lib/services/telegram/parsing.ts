export function parseJsonObject(value: string, field: string): Record<string, unknown> {
	const trimmed = value.trim();
	if (!trimmed) {
		return {};
	}

	const parsed = JSON.parse(trimmed) as unknown;
	if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
		throw new Error(`${field} must be a JSON object`);
	}
	return parsed as Record<string, unknown>;
}

export function parseStringMap(value: string, field: string): Record<string, string> {
	const parsed = parseJsonObject(value, field);
	return Object.fromEntries(
		Object.entries(parsed).map(([key, rawValue]) => {
			if (typeof rawValue !== 'string') {
				throw new Error(`${field}.${key} must be a string`);
			}
			return [key, rawValue];
		})
	);
}
