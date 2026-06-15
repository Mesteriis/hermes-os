export function plainTextToComposeHtml(text: string): string {
	const paragraphs = text
		.split(/\n{2,}/)
		.map((paragraph) => paragraph.trim())
		.filter(Boolean)

	if (paragraphs.length === 0) return '<p></p>'

	return paragraphs
		.map((paragraph) => `<p>${escapeHtml(paragraph).replace(/\n/g, '<br>')}</p>`)
		.join('')
}

export function htmlToComposePlainText(html: string): string {
	return html
		.replace(/<\/(p|div|li|h[1-6])>/gi, '\n')
		.replace(/<br\s*\/?>/gi, '\n')
		.replace(/<[^>]+>/g, '')
		.replace(/&nbsp;/g, ' ')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#39;/g, "'")
		.replace(/&amp;/g, '&')
		.split('\n')
		.map((line) => line.trim())
		.filter(Boolean)
		.join('\n')
}

export function appendPlainTextSignature(body: string, signature: string): string {
	const existingBody = body.trim()
	const trimmedSignature = signature.trim()
	if (!trimmedSignature) return existingBody
	return `${existingBody}${existingBody ? '\n\n' : ''}${trimmedSignature}`
}

export function appendHtmlSignature(bodyHtml: string | null | undefined, signature: string): string {
	const existingHtml = bodyHtml?.trim() ?? ''
	const trimmedSignature = signature.trim()
	if (!trimmedSignature) return existingHtml
	return `${existingHtml}${existingHtml ? '<p></p>' : ''}<p>${escapeHtml(trimmedSignature).replace(/\n/g, '<br>')}</p>`
}

export function normalizeMailComposeLinkHref(value: string): string | null {
	const trimmedValue = value.trim()
	if (!trimmedValue || /\s/.test(trimmedValue)) return null

	const href = /^[a-z][a-z0-9+.-]*:/i.test(trimmedValue)
		? trimmedValue
		: `https://${trimmedValue}`

	try {
		const parsedHref = new URL(href)
		if (!['http:', 'https:', 'mailto:'].includes(parsedHref.protocol)) return null
		if (parsedHref.protocol === 'mailto:' && !parsedHref.pathname) return null
		if (
			(parsedHref.protocol === 'http:' || parsedHref.protocol === 'https:') &&
			(parsedHref.username || parsedHref.password)
		) {
			return null
		}
		return parsedHref.toString()
	} catch {
		return null
	}
}

export type MailComposeTextAlign = 'center' | 'left' | 'right'

export function normalizeMailComposeTextAlign(value: string | null | undefined): MailComposeTextAlign | null {
	const normalizedValue = value?.trim().toLowerCase()
	if (normalizedValue === 'center' || normalizedValue === 'left' || normalizedValue === 'right') {
		return normalizedValue
	}
	return null
}

const SKIPPED_PASTE_CONTENT_TAGS = new Set([
	'head',
	'iframe',
	'link',
	'meta',
	'object',
	'script',
	'style',
	'svg'
])

const NORMALIZED_PASTE_TAGS = new Map([
	['a', 'a'],
	['b', 'strong'],
	['blockquote', 'blockquote'],
	['br', 'br'],
	['div', 'p'],
	['em', 'em'],
	['h1', 'h2'],
	['h2', 'h2'],
	['h3', 'h3'],
	['h4', 'h3'],
	['h5', 'h3'],
	['h6', 'h3'],
	['i', 'em'],
	['li', 'li'],
	['ol', 'ol'],
	['p', 'p'],
	['strong', 'strong'],
	['ul', 'ul']
])

export function sanitizeMailComposePastedHtml(html: string): string {
	const source = html.trim()
	if (!source) return '<p></p>'

	const emittedTags: string[] = []
	let skippedContentTag: string | null = null
	let sanitized = ''

	for (const token of source.split(/(<[^>]*>)/g)) {
		if (!token) continue
		const parsedTag = parseHtmlTag(token)
		if (!parsedTag) {
			if (!skippedContentTag) sanitized += token
			continue
		}

		if (SKIPPED_PASTE_CONTENT_TAGS.has(parsedTag.name)) {
			if (!parsedTag.closing && !parsedTag.selfClosing) skippedContentTag = parsedTag.name
			if (parsedTag.closing && skippedContentTag === parsedTag.name) skippedContentTag = null
			continue
		}
		if (skippedContentTag) continue

		const normalizedTag = NORMALIZED_PASTE_TAGS.get(parsedTag.name)
		if (!normalizedTag) continue
		if (parsedTag.closing) {
			sanitized += closePastedTag(normalizedTag, emittedTags)
			continue
		}
		if (normalizedTag === 'br') {
			sanitized += '<br>'
			continue
		}
		if (normalizedTag === 'a') {
			const href = normalizeMailComposeLinkHref(extractHtmlAttribute(parsedTag.attributes, 'href') ?? '')
			if (!href) continue
			emittedTags.push(normalizedTag)
			sanitized += `<a href="${escapeHtmlAttribute(href)}" rel="noopener noreferrer" target="_blank">`
			continue
		}

		emittedTags.push(normalizedTag)
		sanitized += `<${normalizedTag}${renderPastedTextAlignAttribute(parsedTag.attributes, normalizedTag)}>`
		if (parsedTag.selfClosing) sanitized += closePastedTag(normalizedTag, emittedTags)
	}

	while (emittedTags.length > 0) {
		sanitized += `</${emittedTags.pop()}>`
	}

	return sanitized.trim() || '<p></p>'
}

function parseHtmlTag(value: string): {
	attributes: string
	closing: boolean
	name: string
	selfClosing: boolean
} | null {
	const match = value.match(/^<\s*(\/)?\s*([a-z][a-z0-9-]*)([^>]*)>/i)
	if (!match) return null
	const attributes = match[3] ?? ''
	return {
		attributes,
		closing: Boolean(match[1]),
		name: match[2].toLowerCase(),
		selfClosing: /\/\s*>$/.test(value)
	}
}

function renderPastedTextAlignAttribute(attributes: string, tag: string): string {
	if (!['h2', 'h3', 'p'].includes(tag)) return ''
	const textAlign = normalizeMailComposeTextAlign(extractCssDeclarationValue(attributes, 'text-align'))
	return textAlign ? ` style="text-align: ${textAlign}"` : ''
}

function extractCssDeclarationValue(attributes: string, propertyName: string): string | null {
	const style = extractHtmlAttribute(attributes, 'style')
	if (!style) return null
	const normalizedPropertyName = propertyName.toLowerCase()
	for (const declaration of style.split(';')) {
		const separatorIndex = declaration.indexOf(':')
		if (separatorIndex === -1) continue
		const name = declaration.slice(0, separatorIndex).trim().toLowerCase()
		if (name !== normalizedPropertyName) continue
		return declaration.slice(separatorIndex + 1).trim()
	}
	return null
}

function closePastedTag(tag: string, emittedTags: string[]): string {
	const lastTag = emittedTags.at(-1)
	if (lastTag !== tag) return ''
	emittedTags.pop()
	return `</${tag}>`
}

function extractHtmlAttribute(attributes: string, name: string): string | null {
	const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
	const match = attributes.match(new RegExp(`\\s${escapedName}\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s"'>/]+))`, 'i'))
	return match?.[1] ?? match?.[2] ?? match?.[3] ?? null
}

function escapeHtmlAttribute(value: string): string {
	return escapeHtml(value)
}

function escapeHtml(value: string): string {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;')
}
