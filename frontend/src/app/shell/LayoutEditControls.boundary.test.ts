import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('LayoutEditControls boundary', () => {
  it('delegates layout editor actions to an app-level surface', () => {
    const source = readFileSync(new URL('./LayoutEditControls.vue', import.meta.url), 'utf8')

    expect(source).toContain("import { useLayoutEditControlsSurface } from '../queries/useLayoutEditControlsSurface'")
    expect(source).toContain('const { editor, handleAddWidget, handleCancel, handleReset, handleSave } = useLayoutEditControlsSurface()')
    expect(source).not.toContain('useLayoutEditorStore')
    expect(source).not.toContain('editor.openAddWidgetDrawer')
    expect(source).not.toContain('editor.cancelLayoutEditing')
    expect(source).not.toContain('editor.resetCurrentViewLayout')
    expect(source).not.toContain('editor.saveLayoutSettings')
  })
})
