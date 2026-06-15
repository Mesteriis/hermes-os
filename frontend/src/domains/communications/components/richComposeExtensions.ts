import { Mark, Node } from '@tiptap/vue-3'
import { normalizeMailComposeLinkHref, normalizeMailComposeTextAlign } from './richComposeHtml'

const RichDocument = Node.create({
	name: 'doc',
	topNode: true,
	content: 'block+'
})

const RichParagraph = Node.create({
	name: 'paragraph',
	group: 'block',
	content: 'inline*',
	addAttributes() {
		return {
			textAlign: {
				default: null,
				parseHTML: (element) => normalizeMailComposeTextAlign(element.style.textAlign),
				renderHTML: (attributes) => getSafeTextAlignAttributes(attributes.textAlign)
			}
		}
	},
	parseHTML() {
		return [{ tag: 'p' }]
	},
	renderHTML({ HTMLAttributes }) {
		return ['p', HTMLAttributes, 0]
	}
})

const RichHeading = Node.create({
	name: 'heading',
	group: 'block',
	content: 'inline*',
	addAttributes() {
		return {
			level: {
				default: 2,
				parseHTML: (element) => {
					const level = Number(element.tagName.slice(1))
					return level === 3 ? 3 : 2
				},
				renderHTML: () => ({})
			},
			textAlign: {
				default: null,
				parseHTML: (element) => normalizeMailComposeTextAlign(element.style.textAlign),
				renderHTML: (attributes) => getSafeTextAlignAttributes(attributes.textAlign)
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
		const level = node.attrs.level === 3 ? 3 : 2
		return [`h${level}`, HTMLAttributes, 0]
	}
})

const RichText = Node.create({
	name: 'text',
	group: 'inline'
})

const RichBulletList = Node.create({
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

const RichOrderedList = Node.create({
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

const RichListItem = Node.create({
	name: 'listItem',
	content: 'paragraph block*',
	parseHTML() {
		return [{ tag: 'li' }]
	},
	renderHTML() {
		return ['li', 0]
	}
})

const RichBlockquote = Node.create({
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

const RichBold = Mark.create({
	name: 'bold',
	parseHTML() {
		return [
			{ tag: 'strong' },
			{ tag: 'b' },
			{
				style: 'font-weight',
				getAttrs: (value) => {
					return /^(bold(er)?|[5-9]\d{2,})$/.test(String(value)) ? null : false
				}
			}
		]
	},
	renderHTML() {
		return ['strong', 0]
	}
})

const RichItalic = Mark.create({
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

function getSafeTextAlignAttributes(value: unknown): Record<string, string> {
	if (typeof value !== 'string') return {}
	const textAlign = normalizeMailComposeTextAlign(value)
	return textAlign ? { style: `text-align: ${textAlign}` } : {}
}

function getSafeLinkAttributes(value: unknown): Record<string, string> {
	if (typeof value !== 'string') return {}
	const href = normalizeMailComposeLinkHref(value)
	if (!href) return {}
	return {
		href,
		rel: 'noopener noreferrer',
		target: '_blank'
	}
}

const RichLink = Mark.create({
	name: 'link',
	inclusive: false,
	addAttributes() {
		return {
			href: {
				default: null,
				parseHTML: (element) => normalizeMailComposeLinkHref(element.getAttribute('href') ?? ''),
				renderHTML: (attributes) => getSafeLinkAttributes(attributes.href)
			}
		}
	},
	parseHTML() {
		return [
			{
				tag: 'a[href]',
				getAttrs: (node) => {
					if (!(node instanceof HTMLElement)) return false
					return normalizeMailComposeLinkHref(node.getAttribute('href') ?? '') ? null : false
				}
			}
		]
	},
	renderHTML({ HTMLAttributes }) {
		const attributes = getSafeLinkAttributes(HTMLAttributes.href)
		return Object.keys(attributes).length > 0 ? ['a', attributes, 0] : ['span', 0]
	}
})

export const richComposeExtensions = [
	RichDocument,
	RichParagraph,
	RichHeading,
	RichText,
	RichBulletList,
	RichOrderedList,
	RichListItem,
	RichBlockquote,
	RichBold,
	RichItalic,
	RichLink
]
