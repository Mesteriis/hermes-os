import { useQuery } from '@tanstack/vue-query'
import { fetchEmailAccounts } from '../api/communications'
import type { EmailAccountListResponse } from '../types/communications'
import { communicationReferenceQueryOptions } from './communicationQueryPolicies'

export function useEmailAccountsQuery() {
  return useQuery<EmailAccountListResponse, Error>({
    queryKey: ['communications', 'mail', 'accounts'],
    queryFn: fetchEmailAccounts,
    ...communicationReferenceQueryOptions
  })
}
