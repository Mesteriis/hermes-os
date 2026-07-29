import { useMediaQuery } from '@vueuse/core'
import { ref, watch, type Ref } from 'vue'

const COMPACT_WORKSPACE_QUERY = '(max-width: 760px)'

export function useResponsiveWorkspaceInspector(): Ref<boolean> {
	const compactWorkspace = useMediaQuery(COMPACT_WORKSPACE_QUERY)
	const visible = ref(!compactWorkspace.value)

	watch(compactWorkspace, (compact) => {
		if (compact) {
			visible.value = false
		}
	})

	return visible
}
