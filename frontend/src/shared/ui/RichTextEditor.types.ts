export type RichTextEditorAction =
	| 'paragraph'
	| 'heading'
	| 'subheading'
	| 'quote'
	| 'bulletList'
	| 'orderedList'
	| 'bold'
	| 'italic'
	| 'underline'
	| 'strike'
	| 'code'
	| 'link'
	| 'codeBlock'
	| 'horizontalRule'
	| 'clearFormatting'

export type RichTextEditorToolbarGroup =
	| 'structure'
	| 'lists'
	| 'marks'
	| 'insert'
	| 'cleanup'

export interface RichTextEditorToolbarAction {
	id: RichTextEditorAction
	label: string
	icon: string
	group?: RichTextEditorToolbarGroup
}
