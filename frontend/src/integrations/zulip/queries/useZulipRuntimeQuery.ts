import { useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  enqueueZulipDirectUploadCommand,
  enqueueZulipStreamUploadCommand,
  enqueueZulipUploadCommand,
  setupZulipBotAccount,
} from '../api/zulip'
import type {
  ZulipAccountSetupRequest,
  ZulipAccountSetupResponse,
  ZulipCommandEnqueueResponse,
  ZulipDirectUploadCommandRequest,
  ZulipStreamUploadCommandRequest,
  ZulipUploadCommandRequest,
} from '../types/zulip'
import { zulipQueryKeys } from './zulipQueryKeys'
import { settingsKeys } from '../../../shared/zulip/settingsBridge'

export function useSetupZulipBotAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZulipAccountSetupResponse, Error, ZulipAccountSetupRequest>({
    mutationFn: setupZulipBotAccount,
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: zulipQueryKeys.accounts }),
        queryClient.invalidateQueries({ queryKey: settingsKeys.providerAccounts() }),
        queryClient.invalidateQueries({ queryKey: settingsKeys.workspace() }),
      ])
    },
  })
}

export function useEnqueueZulipStreamUploadCommandMutation(accountId: () => string | null | undefined) {
  return useZulipAccountCommandMutation<ZulipStreamUploadCommandRequest>(
    accountId,
    enqueueZulipStreamUploadCommand
  )
}

export function useEnqueueZulipDirectUploadCommandMutation(accountId: () => string | null | undefined) {
  return useZulipAccountCommandMutation<ZulipDirectUploadCommandRequest>(
    accountId,
    enqueueZulipDirectUploadCommand
  )
}

export function useEnqueueZulipUploadCommandMutation(accountId: () => string | null | undefined) {
  return useZulipAccountCommandMutation<ZulipUploadCommandRequest>(
    accountId,
    enqueueZulipUploadCommand
  )
}

function useZulipAccountCommandMutation<TRequest>(
  accountId: () => string | null | undefined,
  mutationFn: (accountId: string, request: TRequest) => Promise<ZulipCommandEnqueueResponse>
) {
  const queryClient = useQueryClient()
  return useMutation<ZulipCommandEnqueueResponse, Error, TRequest>({
    mutationFn: async (request) => {
      const value = accountId()?.trim()
      if (!value) {
        throw new Error('Zulip account id is required')
      }
      return mutationFn(value, request)
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: zulipQueryKeys.commands })
    },
  })
}
