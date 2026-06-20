import { useMutation } from '@tanstack/vue-query'
import {
  setupImapEmailAccount,
  startGmailOAuthSetup,
  type EmailAccountSetupResponse,
  type GmailOAuthStartRequest,
  type GmailOAuthStartResponse,
  type ImapEmailAccountSetupRequest
} from '../api/accountSetup'

export function useStartGmailOAuthSetupMutation() {
  return useMutation<GmailOAuthStartResponse, Error, GmailOAuthStartRequest>({
    mutationFn: async (request) => startGmailOAuthSetup(request)
  })
}

export function useSetupImapEmailAccountMutation() {
  return useMutation<EmailAccountSetupResponse, Error, ImapEmailAccountSetupRequest>({
    mutationFn: async (request) => setupImapEmailAccount(request)
  })
}
