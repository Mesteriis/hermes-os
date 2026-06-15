<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import {
	EditorContent,
	useEditor,
	type Editor
} from '@tiptap/vue-3'
import Icon from '../../../shared/ui/Icon.vue'
import { richComposeExtensions } from './richComposeExtensions'
import { normalizeMailComposeLinkHref, sanitizeMailComposePastedHtml, type MailComposeTextAlign } from './richComposeHtml'

const props = defineProps<{
	modelValue: string
	placeholder?: string
}>()

const emit = defineEmits<{
	'update:modelValue': [value: string]
	'attachments-dropped': [files: File[]]
	blur: []
}>()

const isFocused = ref(false)
const lastAppliedHtml = ref('')
const linkHref = ref('')

function normalizedHtml(value: string): string {
	return value.trim() ? value : '<p></p>'
}

function emitCurrentHtml(editor: Editor): void {
	const html = editor.getHTML()
	lastAppliedHtml.value = html
	emit('update:modelValue', html)
}

function insertSanitizedClipboardHtml(editor: Editor, html: string): boolean {
	if (!html.trim()) return false
	editor.commands.insertContent(sanitizeMailComposePastedHtml(html))
	return true
}

const editor = useEditor({
	content: normalizedHtml(props.modelValue),
	extensions: richComposeExtensions,
	editorProps: {
		attributes: {
			class: 'rich-compose-prosemirror',
			'aria-multiline': 'true',
			role: 'textbox'
		},
		handlePaste: (_view, event) => {
			const html = event.clipboardData?.getData('text/html') ?? ''
			const currentEditor = editor.value
			if (!currentEditor || !insertSanitizedClipboardHtml(currentEditor, html)) return false
			event.preventDefault()
			return true
		},
		handleDrop: (_view, event) => {
			const files = Array.from(event.dataTransfer?.files ?? [])
			if (files.length > 0) {
				emit('attachments-dropped', files)
				event.preventDefault()
				return true
			}
			const html = event.dataTransfer?.getData('text/html') ?? ''
			const currentEditor = editor.value
			if (!currentEditor || !insertSanitizedClipboardHtml(currentEditor, html)) return false
			event.preventDefault()
			return true
		}
	},
	onCreate: ({ editor }) => {
		lastAppliedHtml.value = editor.getHTML()
	},
	onUpdate: ({ editor }) => {
		emitCurrentHtml(editor)
	},
	onFocus: () => {
		isFocused.value = true
	},
	onBlur: ({ editor }) => {
		isFocused.value = false
		emitCurrentHtml(editor)
		emit('blur')
	}
})

watch(
	() => props.modelValue,
	(value) => {
		const currentEditor = editor.value
		if (!currentEditor) return
		const nextHtml = normalizedHtml(value)
		if (nextHtml === lastAppliedHtml.value) return
		if (isFocused.value) return
		if (currentEditor.getHTML() === nextHtml) return
		currentEditor.commands.setContent(nextHtml, { emitUpdate: false })
		lastAppliedHtml.value = nextHtml
	}
)

onBeforeUnmount(() => {
	editor.value?.destroy()
})

type RichComposeCommand =
	| 'alignCenter'
	| 'alignLeft'
	| 'alignRight'
	| 'blockquote'
	| 'bold'
	| 'bulletList'
	| 'heading2'
	| 'heading3'
	| 'italic'
	| 'link'
	| 'orderedList'
	| 'paragraph'
	| 'unlink'

type RichComposeActiveCommand = Exclude<RichComposeCommand, 'unlink'>

function runCommand(command: RichComposeCommand): void {
	const currentEditor = editor.value
	if (!currentEditor) return
	if (command === 'bold') {
		currentEditor.chain().focus().toggleMark('bold').run()
		return
	}
	if (command === 'italic') {
		currentEditor.chain().focus().toggleMark('italic').run()
		return
	}
	if (command === 'paragraph') {
		currentEditor.chain().focus().setNode('paragraph').run()
		return
	}
	if (command === 'heading2') {
		currentEditor.chain().focus().toggleNode('heading', 'paragraph', { level: 2 }).run()
		return
	}
	if (command === 'heading3') {
		currentEditor.chain().focus().toggleNode('heading', 'paragraph', { level: 3 }).run()
		return
	}
	if (command === 'alignLeft') {
		setActiveBlockTextAlign(currentEditor, 'left')
		return
	}
	if (command === 'alignCenter') {
		setActiveBlockTextAlign(currentEditor, 'center')
		return
	}
	if (command === 'alignRight') {
		setActiveBlockTextAlign(currentEditor, 'right')
		return
	}
	if (command === 'orderedList') {
		currentEditor.chain().focus().toggleList('orderedList', 'listItem').run()
		return
	}
	if (command === 'blockquote') {
		currentEditor.chain().focus().toggleWrap('blockquote').run()
		return
	}
	if (command === 'link') {
		const href = normalizeMailComposeLinkHref(linkHref.value)
		if (!href) return
		currentEditor.chain().focus().extendMarkRange('link').setMark('link', { href }).run()
		linkHref.value = href
		return
	}
	if (command === 'unlink') {
		currentEditor.chain().focus().extendMarkRange('link').unsetMark('link').run()
		linkHref.value = ''
		return
	}
	currentEditor.chain().focus().toggleList('bulletList', 'listItem').run()
}

function setActiveBlockTextAlign(editor: Editor, textAlign: MailComposeTextAlign): void {
	const nodeName = editor.isActive('heading') ? 'heading' : 'paragraph'
	editor.chain().focus().updateAttributes(nodeName, { textAlign }).run()
}

function isCommandActive(command: RichComposeActiveCommand): boolean {
	const currentEditor = editor.value
	if (!currentEditor) return false
	if (command === 'heading2') return currentEditor.isActive('heading', { level: 2 })
	if (command === 'heading3') return currentEditor.isActive('heading', { level: 3 })
	if (command === 'alignLeft') return isActiveTextAlign(currentEditor, 'left')
	if (command === 'alignCenter') return isActiveTextAlign(currentEditor, 'center')
	if (command === 'alignRight') return isActiveTextAlign(currentEditor, 'right')
	return currentEditor.isActive(command)
}

function isActiveTextAlign(editor: Editor, textAlign: MailComposeTextAlign): boolean {
	return editor.isActive('heading', { textAlign }) || editor.isActive('paragraph', { textAlign })
}
</script>

<template>
	<div class="rich-compose-editor">
		<div class="rich-compose-toolbar" aria-label="Rich text formatting">
			<button type="button" title="Paragraph" :class="{ active: isCommandActive('paragraph') }" @click="runCommand('paragraph')">
				<Icon icon="tabler:pilcrow" size="16" />
			</button>
			<button type="button" title="Heading" :class="{ active: isCommandActive('heading2') }" @click="runCommand('heading2')">
				<Icon icon="tabler:h-2" size="16" />
			</button>
			<button type="button" title="Subheading" :class="{ active: isCommandActive('heading3') }" @click="runCommand('heading3')">
				<Icon icon="tabler:h-3" size="16" />
			</button>
			<button type="button" title="Align left" :class="{ active: isCommandActive('alignLeft') }" @click="runCommand('alignLeft')">
				<Icon icon="tabler:align-left" size="16" />
			</button>
			<button type="button" title="Align center" :class="{ active: isCommandActive('alignCenter') }" @click="runCommand('alignCenter')">
				<Icon icon="tabler:align-center" size="16" />
			</button>
			<button type="button" title="Align right" :class="{ active: isCommandActive('alignRight') }" @click="runCommand('alignRight')">
				<Icon icon="tabler:align-right" size="16" />
			</button>
			<button type="button" title="Bold" :class="{ active: isCommandActive('bold') }" @click="runCommand('bold')">
				<Icon icon="tabler:bold" size="16" />
			</button>
			<button type="button" title="Italic" :class="{ active: isCommandActive('italic') }" @click="runCommand('italic')">
				<Icon icon="tabler:italic" size="16" />
			</button>
			<button type="button" title="Bulleted list" :class="{ active: isCommandActive('bulletList') }" @click="runCommand('bulletList')">
				<Icon icon="tabler:list" size="16" />
			</button>
			<button type="button" title="Numbered list" :class="{ active: isCommandActive('orderedList') }" @click="runCommand('orderedList')">
				<Icon icon="tabler:list-numbers" size="16" />
			</button>
			<button type="button" title="Quote" :class="{ active: isCommandActive('blockquote') }" @click="runCommand('blockquote')">
				<Icon icon="tabler:quote" size="16" />
			</button>
			<span class="rich-compose-link-tools">
				<input
					v-model="linkHref"
					type="url"
					placeholder="https://example.com"
					aria-label="Link URL"
					@keydown.enter.prevent="runCommand('link')"
				>
				<button type="button" title="Link" :class="{ active: isCommandActive('link') }" @click="runCommand('link')">
					<Icon icon="tabler:link" size="16" />
				</button>
				<button type="button" title="Unlink" @click="runCommand('unlink')">
					<Icon icon="tabler:unlink" size="16" />
				</button>
			</span>
		</div>
		<EditorContent
			v-if="editor"
			:editor="editor"
			class="rich-compose-surface"
			:data-placeholder="placeholder ?? 'Write your message...'"
		/>
	</div>
</template>

<style scoped>
.rich-compose-editor {
	display: flex;
	min-height: 240px;
	flex: 1;
	flex-direction: column;
	overflow: hidden;
	border: 1px solid var(--hh-border, #e5e7eb);
	border-radius: 0.375rem;
	background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
}

.rich-compose-editor:focus-within {
	border-color: var(--hh-accent, #3b82f6);
	box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.rich-compose-toolbar {
	display: flex;
	align-items: center;
	gap: 0.125rem;
	padding: 0.25rem;
	border-bottom: 1px solid var(--hh-border, #e5e7eb);
	background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 86%, transparent);
}

.rich-compose-toolbar button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 1.75rem;
	height: 1.75rem;
	border: 0;
	border-radius: 0.25rem;
	background: transparent;
	color: var(--hh-text-secondary, #6b7280);
	cursor: pointer;
}

.rich-compose-toolbar button:hover {
	background: var(--hh-hover-bg, rgba(148, 163, 184, 0.12));
	color: var(--hh-text-primary, #1f2937);
}

.rich-compose-toolbar button.active {
	background: var(--hh-bg-primary, #ffffff);
	color: var(--hh-text-primary, #1f2937);
	box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
}

.rich-compose-link-tools {
	display: inline-flex;
	align-items: center;
	gap: 0.125rem;
	margin-left: 0.25rem;
	padding-left: 0.25rem;
	border-left: 1px solid var(--hh-border, #e5e7eb);
}

.rich-compose-link-tools input {
	width: clamp(9rem, 18vw, 14rem);
	height: 1.75rem;
	border: 1px solid var(--hh-border, #e5e7eb);
	border-radius: 0.25rem;
	background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
	color: var(--hh-text-primary, #1f2937);
	font-size: 0.75rem;
	outline: none;
	padding: 0 0.5rem;
}

.rich-compose-link-tools input:focus {
	border-color: var(--hh-accent, #3b82f6);
	box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.rich-compose-surface {
	display: flex;
	flex: 1;
	min-height: 200px;
	overflow-y: auto;
	position: relative;
}

.rich-compose-surface :deep(.rich-compose-prosemirror) {
	width: 100%;
	min-height: 200px;
	padding: 0.625rem 0.75rem;
	color: var(--hh-text-primary, #1f2937);
	font-size: 0.8125rem;
	line-height: 1.6;
	outline: none;
}

.rich-compose-surface:has(.rich-compose-prosemirror > p:first-child:last-child:empty)::before {
	content: attr(data-placeholder);
	position: absolute;
	top: 0.625rem;
	left: 0.75rem;
	color: var(--hh-text-muted, #9ca3af);
	pointer-events: none;
}

.rich-compose-surface :deep(p) {
	margin: 0 0 0.75rem;
}

.rich-compose-surface :deep(h2) {
	margin: 0 0 0.75rem;
	font-size: 1rem;
	font-weight: 700;
	line-height: 1.4;
}

.rich-compose-surface :deep(h3) {
	margin: 0 0 0.625rem;
	font-size: 0.875rem;
	font-weight: 700;
	line-height: 1.45;
}

.rich-compose-surface :deep(ul) {
	margin: 0 0 0.75rem;
	padding-left: 1.25rem;
}

.rich-compose-surface :deep(ol) {
	margin: 0 0 0.75rem;
	padding-left: 1.25rem;
}

.rich-compose-surface :deep(blockquote) {
	margin: 0 0 0.75rem;
	padding-left: 0.75rem;
	border-left: 2px solid var(--hh-border, #e5e7eb);
	color: var(--hh-text-secondary, #6b7280);
}

.rich-compose-surface :deep(a) {
	color: var(--hh-accent, #3b82f6);
	text-decoration: underline;
	text-underline-offset: 0.125rem;
}
</style>
