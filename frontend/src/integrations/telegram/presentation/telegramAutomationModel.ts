export type TelegramAutomationTemplateRow = {
	id: string
	name: string
	revision: string
}

export type TelegramAutomationPolicyRow = {
	id: string
	name: string
	accountId: string
	enabled: boolean
	revision: string
}

export type TelegramAutomationModel = {
	canCommand: boolean
	canQuery: boolean
	pending: boolean
	statusMessage: string
	templates: readonly TelegramAutomationTemplateRow[]
	policies: readonly TelegramAutomationPolicyRow[]
	template: {
		id: string
		name: string
		body: string
		requiredVariables: string
		revision: string
	}
	policy: {
		id: string
		templateId: string
		name: string
		enabled: boolean
		accountId: string
		providerChatIds: string
		expiresAtUnixSeconds: string
		revision: string
	}
	preview: {
		policyId: string
		accountId: string
		providerChatId: string
		variables: string
		renderedText: string
		renderedSha256: string
	}
}

export function parseAutomationIdentifiers(value: string): string[] {
	const identifiers = value
		.split(/[\s,]+/)
		.map((item) => item.trim())
		.filter(Boolean)
	if (new Set(identifiers).size !== identifiers.length) {
		throw new RangeError('Telegram automation identifiers must be unique')
	}
	return identifiers
}

export function parseAutomationVariables(
	value: string,
): readonly { name: string; value: string }[] {
	const variables = value
		.split('\n')
		.map((line) => line.trim())
		.filter(Boolean)
		.map((line) => {
			const separator = line.indexOf('=')
			if (separator < 1) {
				throw new RangeError('Telegram automation variables use name=value lines')
			}
			return {
				name: line.slice(0, separator).trim(),
				value: line.slice(separator + 1).trim(),
			}
		})
	if (new Set(variables.map(({ name }) => name)).size !== variables.length) {
		throw new RangeError('Telegram automation variable names must be unique')
	}
	return variables
}

export function automationDigestHex(bytes: Uint8Array): string {
	return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('')
}
