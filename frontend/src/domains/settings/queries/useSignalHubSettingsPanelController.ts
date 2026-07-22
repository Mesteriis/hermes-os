import { computed } from 'vue'
import type { useSignalHubSettingsSurface } from './useSignalHubSettingsSurface'
import { signalHubViewPresentation } from '../components/signalHubSettingsPresentation'

type SignalHubSettingsSurface = ReturnType<typeof useSignalHubSettingsSurface>

export function useSignalHubSettingsPanelController(options: {
  surface: SignalHubSettingsSurface
}) {
  const activeSignalViewPresentation = computed(() =>
    signalHubViewPresentation(options.surface.activeSignalView.value)
  )

  return {
    activeSignalViewPresentation,
    handleSelectGraphSource: options.surface.handleSelectGraphSource,
    handleSelectInventorySource: options.surface.handleSelectInventorySource,
    handleSelectSignalView: options.surface.handleSelectSignalView,
    handlePauseSourceSignals: options.surface.handlePauseSourceSignals,
    handleResumeSourceSignals: options.surface.handleResumeSourceSignals,
    handleMuteSourceSignals: options.surface.handleMuteSourceSignals,
    handleUnmuteSourceSignals: options.surface.handleUnmuteSourceSignals,
    handleDisableSource: options.surface.handleDisableSource,
    handleEnableSource: options.surface.handleEnableSource,
  }
}
