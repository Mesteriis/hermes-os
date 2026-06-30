import { useLayoutEditorStore } from '../../shared/stores/layoutEditor'

export function useLayoutEditControlsSurface() {
  const editor = useLayoutEditorStore()

  function handleAddWidget(): void {
    editor.openAddWidgetDrawer()
  }

  function handleCancel(): void {
    editor.cancelLayoutEditing()
  }

  function handleReset(): void {
    editor.resetCurrentViewLayout()
  }

  function handleSave(): void {
    editor.saveLayoutSettings()
  }

  return {
    editor,
    handleAddWidget,
    handleCancel,
    handleReset,
    handleSave
  }
}
