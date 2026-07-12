import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { importMailFile, type MailImportResult } from '../api/mailImportApi'
import type { MailImportKind } from '../forms/mailImport'

export function useMailImportMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailImportResult, Error, { accountId: string; kind: MailImportKind; contentBase64: string }>({
    mutationFn: ({ accountId, kind, contentBase64 }) => importMailFile(accountId, kind, contentBase64),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['communications-list'] }),
        queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] }),
        queryClient.invalidateQueries({ queryKey: ['communications-threads'] })
      ])
    }
  })
}
