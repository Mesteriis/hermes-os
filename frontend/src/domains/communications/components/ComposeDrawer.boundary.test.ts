import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'ComposeDrawer.vue')

describe('ComposeDrawer boundaries', () => {
	it('uses communications query mutations instead of calling the API client directly', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).not.toContain("../api/communications")
		expect(source).toContain('useSendMailMutation')
		expect(source).toContain('useSaveDraftMutation')
		expect(source).toContain('useDeleteDraftMutation')
		expect(source).toContain('useComposeDraftAutosave')
	})

	it('uses the shared Reka-backed Sheet primitive instead of a custom overlay', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain("import Sheet from '../../../shared/ui/Sheet.vue'")
		expect(source).toContain('<Sheet')
		expect(source).toContain('content-class="compose-drawer"')
		expect(source).toContain('@update:open="handleSheetOpenChange"')
		expect(source).not.toContain('compose-drawer-overlay')
		expect(source).not.toContain('@click.self="handleClose"')
	})

	it('uses a dedicated rich editor for HTML compose while preserving source mode', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('RichComposeEditor')
		expect(source).toContain("setBodyFormat('html', 'rich')")
		expect(source).toContain("setBodyFormat('html', 'source')")
		expect(source).toContain('html-body-editor')
	})

	it('delegates template selection and rendering to the template picker component', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('ComposeTemplatePicker')
		expect(source).toContain('@apply="applyRenderedTemplate"')
		expect(source).toContain('@saved=')
		expect(source).toContain('@deleted=')
	})

	it('delegates persona signature selection to the signature picker component', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('ComposeSignaturePicker')
		expect(source).toContain('@apply="applySignature"')
	})

	it('stages dropped compose attachments without pretending provider send supports them', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain('stagedAttachments')
		expect(source).toContain('handleAttachmentFiles')
		expect(source).toContain('attachmentInput')
		expect(source).toContain('@attachments-dropped="handleAttachmentFiles"')
		expect(source).toContain('Attachment upload is not connected to provider send yet')
		expect(source).toContain('removeStagedAttachment')
	})

	it('keeps compose styling in the component-owned stylesheet', () => {
		const source = readFileSync(componentPath, 'utf8')

		expect(source).toContain("import './ComposeDrawer.css'")
		expect(source).not.toContain('<style scoped>')
	})
})
