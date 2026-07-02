import DOMPurify from 'dompurify'
import hljs from 'highlight.js'
import { marked } from 'marked'

export function renderMarkdownToSafeHtml(source: string): string {
	if (!source.trim()) {
		return ''
	}

	const parsed = marked.parse(source, {
		async: false,
		breaks: true,
		gfm: true
	}) as string

	return sanitizeHtml(parsed)
}

export function sanitizeHtml(source: string): string {
	return DOMPurify.sanitize(source, {
		USE_PROFILES: { html: true }
	})
}

export function highlightCodeToSafeHtml(code: string, language: string): string {
	const highlighted = hljs.getLanguage(language)
		? hljs.highlight(code, { language, ignoreIllegals: true }).value
		: hljs.highlightAuto(code).value

	return DOMPurify.sanitize(highlighted, {
		ALLOWED_TAGS: ['span'],
		ALLOWED_ATTR: ['class']
	})
}
