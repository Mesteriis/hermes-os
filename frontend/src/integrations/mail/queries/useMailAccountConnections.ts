import { computed, shallowRef } from 'vue'

import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import type { MailAccountStatusV1 } from '../../../gen/hermes/mail/account/v1/client_pb'
import { listMailAccounts } from '../api/mailAccountQueryClient'
import {
	mailAccountConnections,
	type MailAccountConnection,
} from './mailAccountConnections'

export function useMailAccountConnections(input: {
	canQuery: () => boolean
	modules: () => readonly ClientModuleBootstrapV1[]
}) {
	const accounts = shallowRef<readonly MailAccountStatusV1[]>([])
	const connections = computed<readonly MailAccountConnection[]>(() =>
		mailAccountConnections(input.modules(), accounts.value)
	)

	async function refresh(): Promise<void> {
		accounts.value = []
		if (!input.canQuery()) {
			return
		}
		const catalog = await listMailAccounts()
		accounts.value = catalog.accounts
	}

	return { connections, refresh }
}
