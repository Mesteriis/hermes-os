<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../platform/i18n'
import type { WhatsappAccountSummary } from '../../../shared/communications/types/whatsapp'

const props = defineProps<{
	accounts: WhatsappAccountSummary[]
	selectedAccountId: string | null
	includeRemovedAccounts: boolean
}>()

const emit = defineEmits<{
	(event: 'update:includeRemovedAccounts', value: boolean): void
	(event: 'select-account', accountId: string): void
}>()

const { t } = useI18n()

const includeRemovedAccountsModel = computed({
	get: () => props.includeRemovedAccounts,
	set: (value: boolean) => emit('update:includeRemovedAccounts', value),
})
</script>

<template>
	<section class="panel runtime-card">
		<header>
			<h2>{{ t('Accounts') }}</h2>
			<span>{{ accounts.length }}</span>
		</header>
		<label class="runtime-field compact checkbox-field">
			<span>{{ t('Include removed') }}</span>
			<input v-model="includeRemovedAccountsModel" type="checkbox" />
		</label>
		<div v-if="accounts.length" class="account-list">
			<button
				v-for="account in accounts"
				:key="account.account_id"
				type="button"
				class="account-row"
				:data-selected="account.account_id === selectedAccountId"
				@click="emit('select-account', account.account_id)"
			>
				<strong>{{ account.display_name }}</strong>
				<span>{{ account.account_id }}</span>
				<small>
					{{ account.provider_shape ?? account.provider_kind }}
					· {{ account.lifecycle_state ?? 'unknown' }}
				</small>
			</button>
		</div>
		<p v-else class="empty-state">{{ t('No WhatsApp accounts configured yet.') }}</p>
	</section>
</template>
