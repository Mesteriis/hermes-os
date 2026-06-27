import { stringValue } from '../../../shared/communications/queries/realtimePatchShared'

export type WhatsAppRuntimeEventPayload = Record<string, unknown>

export function integerValue(value: unknown): number | null {
	return typeof value === 'number' && Number.isInteger(value) ? value : null
}

export function booleanValue(value: unknown): boolean | null {
	return typeof value === 'boolean' ? value : null
}

export function stringArray(value: unknown): string[] | null {
	return Array.isArray(value) && value.every((item) => typeof item === 'string')
		? [...value]
		: null
}

export function nullableStringValue(value: unknown, fallback: string | null): string | null {
	if (value === null) return null
	return stringValue(value) ?? fallback
}
