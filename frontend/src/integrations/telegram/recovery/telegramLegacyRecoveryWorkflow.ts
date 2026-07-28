import { Code, ConnectError } from '@connectrpc/connect'

import type { TelegramAccountResponse } from '../../../gen/hermes/telegram/v1/client_pb'
import {
	createLegacyProviderRecoveryHostV1,
	legacyRecoveryOperationIdV1,
	type LegacyProviderRecoveryCandidateV1,
	type LegacyProviderRecoveryHostV1,
	type LegacyProviderRecoveryPlanV1,
} from '../../../platform/legacy-recovery'
import { ManagedIntegrationSetupV1 } from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
	type SanitizedProvisioningHostReceiptV1,
} from '../../../platform/vault'
import {
	listTelegramAccounts,
	provisionTelegramAccount,
} from '../api/telegramLifecycleGateway'
import { withTelegramConfigurationRuntimeV1 } from '../setup/telegramConfigurationRuntimeRetry'

const TELEGRAM_STORAGE_CAPABILITY_ID = 'telegram.storage.v1'
const API_HASH_PROVISIONING_CAPABILITY_ID =
	'telegram.api-hash.credential-provisioning.v1'
const SESSION_KEY_PROVISIONING_CAPABILITY_ID =
	'telegram.session-store-key.credential-provisioning.v1'
const RECOVERY_CREDENTIAL_REVISION = 1n

type TelegramLegacyRecoveryPortsV1 = {
	source: LegacyProviderRecoveryHostV1
	configuration: Pick<ManagedIntegrationSetupV1, 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provisionCustodied'>
	lifecycle: {
		list(): Promise<readonly TelegramAccountResponse[]>
		provision(input: {
			accountId: string
			displayName: string
			externalAccountId: string
			credentials: readonly { purpose: string; revision: bigint }[]
		}): Promise<TelegramAccountResponse>
	}
}

export type TelegramLegacyRecoveryResultV1 = {
	state: 'qr_authorization_required'
	apiHash?: SanitizedProvisioningHostReceiptV1
	sessionKey?: SanitizedProvisioningHostReceiptV1
}

export class TelegramLegacyRecoveryWorkflowV1 {
	constructor(private readonly ports: TelegramLegacyRecoveryPortsV1 = defaultPorts()) {}

	async recover(input: {
		registrationId: string
		expectedDesiredRevision: bigint
		plan: LegacyProviderRecoveryPlanV1
		candidate: LegacyProviderRecoveryCandidateV1
	}): Promise<TelegramLegacyRecoveryResultV1> {
		if (input.candidate.kind !== 'telegram_user') {
			throw new Error('Telegram legacy recovery candidate is invalid')
		}
		const source = await this.ports.source.source(
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
		)
		if (source.kind !== 'telegram_user'
			|| source.sourceHandle !== input.candidate.sourceHandle) {
			throw new Error('Telegram legacy recovery source is invalid')
		}
		const operation = (step: string) => legacyRecoveryOperationIdV1(
			input.plan.bundleFingerprintSha256,
			input.candidate.sourceHandle,
			`telegram_${step}`,
		)
		const configurationInstanceId = input.registrationId
		const nextDesiredRevision = input.expectedDesiredRevision + 1n
		await this.ports.configuration.apply({
			registrationId: input.registrationId,
			expectedDesiredRevision: input.expectedDesiredRevision,
			storageCapabilityId: TELEGRAM_STORAGE_CAPABILITY_ID,
			configurationInstanceId,
			requestHostBridge: false,
			values: [
				{
					settingId: 'telegram.account_id',
					value: { case: 'stringValue', value: source.accountId },
				},
				{
					settingId: 'telegram.api_id',
					value: { case: 'signedIntegerValue', value: source.apiId },
				},
			],
			updateOperationId: await operation(
				`update_settings_revision_${input.expectedDesiredRevision}`,
			),
			applyOperationId: await operation(`apply_settings_revision_${nextDesiredRevision}`),
		})
		const existing = await withTelegramConfigurationRuntimeV1(
			() => this.ports.lifecycle.list(),
		)
		if (existing.some((account) => account.accountId === source.accountId)) {
			return { state: 'qr_authorization_required' }
		}
		const apiHash = await this.ports.vault.provisionCustodied({
			operationId: await operation('provision_api_hash'),
			targetRegistrationId: input.registrationId,
			capabilityId: API_HASH_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId,
			purposeId: 'telegram_api_hash',
			secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
		}, (authorized) => this.ports.source.sealSource({
			...authorized,
			recoverySessionId: input.plan.recoverySessionId,
			sourceHandle: input.candidate.sourceHandle,
			secretPurpose: 'telegram_api_hash',
		}))
		const credentialsWithoutNewSession = credentials(
			apiHash.secretRevision,
			RECOVERY_CREDENTIAL_REVISION,
		)
		try {
			await this.ports.lifecycle.provision({
				accountId: source.accountId,
				displayName: source.displayName,
				externalAccountId: source.externalAccountId,
				credentials: credentialsWithoutNewSession,
			})
			return { state: 'qr_authorization_required', apiHash }
		} catch (error) {
			if (!isRecoverableMissingSessionKey(error)) throw error
		}
		const sessionKey = await this.ports.vault.provisionCustodied({
			operationId: await operation('provision_session_store_key'),
			targetRegistrationId: input.registrationId,
			capabilityId: SESSION_KEY_PROVISIONING_CAPABILITY_ID,
			configurationInstanceId,
			purposeId: 'telegram_session_store_key',
			secretClass: OwnerVaultSecretClassV1.SESSION_STORE_KEY,
			action: OwnerVaultActionV1.CREATE,
			secretRevision: 1n,
		}, (authorized) => this.ports.source.sealSource({
			...authorized,
			recoverySessionId: input.plan.recoverySessionId,
			sourceHandle: input.candidate.sourceHandle,
			secretPurpose: 'generated_telegram_session_store_key',
		}))
		await withTelegramConfigurationRuntimeV1(() =>
			this.ports.lifecycle.provision({
			accountId: source.accountId,
			displayName: source.displayName,
			externalAccountId: source.externalAccountId,
			credentials: credentials(apiHash.secretRevision, sessionKey.secretRevision),
			}),
		)
		return { state: 'qr_authorization_required', apiHash, sessionKey }
	}
}

function credentials(
	apiHashRevision: bigint,
	sessionKeyRevision: bigint,
): readonly { purpose: string; revision: bigint }[] {
	return [
		{ purpose: 'telegram_api_hash', revision: apiHashRevision },
		{ purpose: 'telegram_session_encryption_key', revision: sessionKeyRevision },
	]
}

function isRecoverableMissingSessionKey(error: unknown): boolean {
	if (error instanceof ConnectError) {
		return error.code === Code.Unknown
			|| error.code === Code.Unavailable
			|| error.code === Code.FailedPrecondition
	}
	return error instanceof Error
		&& /credential|runtime unavailable|RUNTIME_UNAVAILABLE/i.test(error.message)
}

function defaultPorts(): TelegramLegacyRecoveryPortsV1 {
	return {
		source: createLegacyProviderRecoveryHostV1(),
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		lifecycle: {
			list: listTelegramAccounts,
			provision: provisionTelegramAccount,
		},
	}
}
