import type {
  SignalHubProfileCreateRequest,
  SignalHubProfileUpdateRequest
} from '../types/signalHub'

interface SaveProfileDependencies {
  update: (variables: {
    profileCode: string
    request: SignalHubProfileUpdateRequest
  }) => Promise<unknown>
  create: (request: SignalHubProfileCreateRequest) => Promise<unknown>
}

export function saveSignalHubProfile(
  selectedProfileCode: string | null,
  profileCode: string,
  request: Pick<SignalHubProfileCreateRequest, 'display_name' | 'description' | 'source_policies'>,
  dependencies: SaveProfileDependencies
): Promise<unknown> {
  if (selectedProfileCode) {
    return dependencies.update({ profileCode: selectedProfileCode, request })
  }
  return dependencies.create({ code: profileCode, ...request })
}

interface RemoveProfileDependencies {
  remove: (profileCode: string) => Promise<unknown>
  resetEditor: () => void
}

export async function removeSignalHubProfile(
  profileCode: string,
  dependencies: RemoveProfileDependencies
): Promise<void> {
  await dependencies.remove(profileCode)
  dependencies.resetEditor()
}
