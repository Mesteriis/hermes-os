import { Mark, Node } from '@tiptap/vue-3'

const RichTextDocument = Node.create({
	name: 'doc',
	topNode: true,
	content: 'block+'
})

const RichTextParagraph = Node.create({
	name: 'paragraph',
	group: 'block',
	content: 'inline*',
	parseHTML() {
		return [{ tag: 'p' }]
	},
	renderHTML({ HTMLAttributes }) {
		return ['p', HTMLAttributes, 0]
	}
})

const RichTextHeading = Node.create({
	name: 'heading',
	group: 'block',
	content: 'inline*',
	addAttributes() {
		return {
			level: {
				default: 2,
				parseHTML: (element) => Number(element.tagName.slice(1)) === 3 ? 3 : 2,
				renderHTML: () => ({})
			}
		}
	},
	parseHTML() {
		return [
			{ tag: 'h2' },
			{ tag: 'h3' }
		]
	},
	renderHTML({ HTMLAttributes, node }) {
		return [`h${node.attrs.level === 3 ? 3 : 2}`, HTMLAttributes, 0]
	}
})

const RichTextText = Node.create({
	name: 'text',
	group: 'inline'
})

const RichTextBulletList = Node.create({
	name: 'bulletList',
	group: 'block',
	content: 'listItem+',
	parseHTML() {
		return [{ tag: 'ul' }]
	},
	renderHTML() {
		return ['ul', 0]
	}
})

const RichTextOrderedList = Node.create({
	name: 'orderedList',
	group: 'block',
	content: 'listItem+',
	parseHTML() {
		return [{ tag: 'ol' }]
	},
	renderHTML() {
		return ['ol', 0]
	}
})

const RichTextListItem = Node.create({
	name: 'listItem',
	content: 'paragraph block*',
	parseHTML() {
		return [{ tag: 'li' }]
	},
	renderHTML() {
		return ['li', 0]
	}
})

const RichTextBlockquote = Node.create({
	name: 'blockquote',
	group: 'block',
	content: 'block+',
	parseHTML() {
		return [{ tag: 'blockquote' }]
	},
	renderHTML() {
		return ['blockquote', 0]
	}
})

const RichTextCodeBlock = Node.create({
	name: 'codeBlock',
	group: 'block',
	content: 'text*',
	code: true,
	defining: true,
	parseHTML() {
		return [{ tag: 'pre', preserveWhitespace: 'full' }]
	},
	renderHTML() {
		return ['pre', ['code', 0]]
	}
})

const RichTextHorizontalRule = Node.create({
	name: 'horizontalRule',
	group: 'block',
	parseHTML() {
		return [{ tag: 'hr' }]
	},
	renderHTML() {
		return ['hr']
	}
})

const RichTextBold = Mark.create({
	name: 'bold',
	parseHTML() {
		return [
			{ tag: 'strong' },
			{ tag: 'b' },
			{
				style: 'font-weight',
				getAttrs: (value) => /^(bold(er)?|[5-9]\d{2,})$/.test(String(value)) ? null : false
			}
		]
	},
	renderHTML() {
		return ['strong', 0]
	}
})

const RichTextItalic = Mark.create({
	name: 'italic',
	parseHTML() {
		return [
			{ tag: 'em' },
			{ tag: 'i' },
			{ style: 'font-style=italic' }
		]
	},
	renderHTML() {
		return ['em', 0]
	}
})

const RichTextUnderline = Mark.create({
	name: 'underline',
	parseHTML() {
		return [
			{ tag: 'u' },
			{
				style: 'text-decoration',
				getAttrs: (value) => String(value).includes('underline') ? null : false
			}
		]
	},
	renderHTML() {
		return ['u', 0]
	}
})

const RichTextStrike = Mark.create({
	name: 'strike',
	parseHTML() {
		return [
			{ tag: 's' },
			{ tag: 'del' },
			{ tag: 'strike' },
			{
				style: 'text-decoration',
				getAttrs: (value) => String(value).includes('line-through') ? null : false
			}
		]
	},
	renderHTML() {
		return ['s', 0]
	}
})

const RichTextCode = Mark.create({
	name: 'code',
	code: true,
	excludes: '_',
	parseHTML() {
		return [{ tag: 'code' }]
	},
	renderHTML() {
		return ['code', 0]
	}
})

const RichTextLink = Mark.create({
	name: 'link',
	inclusive: false,
	addAttributes() {
		return {
			href: {
				default: null,
				parseHTML: (element) => normalizeRichTextHref(element.getAttribute('href') ?? ''),
				renderHTML: (attributes) => {
					const href = normalizeRichTextHref(attributes.href)
					return href
						? { href, rel: 'noopener noreferrer', target: '_blank' }
						: {}
				}
			}
		}
	},
	parseHTML() {
		return [
			{
				tag: 'a[href]',
				getAttrs: (node) => {
					if (!(node instanceof HTMLElement)) return false
					return normalizeRichTextHref(node.getAttribute('href') ?? '') ? null : false
				}
			}
		]
	},
	renderHTML({ HTMLAttributes }) {
		const href = normalizeRichTextHref(HTMLAttributes.href)
		return href
			? ['a', { href, rel: 'noopener noreferrer', target: '_blank' }, 0]
			: ['span', 0]
	}
})

export const richTextEditorExtensions = [
	RichTextDocument,
	RichTextParagraph,
	RichTextHeading,
	RichTextText,
	RichTextBulletList,
	RichTextOrderedList,
	RichTextListItem,
	RichTextBlockquote,
	RichTextCodeBlock,
	RichTextHorizontalRule,
	RichTextBold,
	RichTextItalic,
	RichTextUnderline,
	RichTextStrike,
	RichTextCode,
	RichTextLink
]

export function normalizeRichTextHref(value: unknown): string | null {
	if (typeof value !== 'string') return null
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
