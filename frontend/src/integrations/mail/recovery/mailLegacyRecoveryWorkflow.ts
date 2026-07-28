import {
	MailAccountReadinessV1,
	MailCredentialPurposeV1,
	type MailAccountStatusV1,
	type MailCredentialBindingReceiptV1,
} from '../../../gen/hermes/mail/account/v1/client_pb'
import type {
	GmailOAuthStartedV1,
	MailAcceptedV1,
} from '../../../gen/hermes/mail/v1/client_pb'
import {
	createLegacyProviderRecoveryHostV1,
	legacyRecoveryOperationIdV1,
	legacyRecoveryOperationKeyV1,
	type LegacyProviderRecoveryCandidateV1,
	type LegacyProviderRecoveryHostV1,
	type LegacyProviderRecoveryPlanV1,
} from '../../../platform/legacy-recovery'
import { ManagedIntegrationSetupV1 } from '../../../platform/settings'
import {
	OwnerVaultActionV1,
	OwnerVaultProvisioningClientV1,
	OwnerVaultSecretClassV1,
} from '../../../platform/vault'
import { bindMailCredential } from '../api/mailCredentialBindingClient'
import { getMailAccountStatus } from '../api/mailAccountQueryClient'
import { MailGmailOAuthClientV1 } from '../api/mailGmailOAuthClient'
import {
	mailGmailPreauthorizationSettings,
	mailImapSettings,
} from '../setup/mailAccountSetupWorkflow'

const MAIL_STORAGE_CAPABILITY_ID = 'mail.storage.v1'

type MailLegacyRecoveryPortsV1 = {
	source: LegacyProviderRecoveryHostV1
	configuration: Pick<ManagedIntegrationSetupV1, 'createTarget' | 'apply'>
	vault: Pick<OwnerVaultProvisioningClientV1, 'provisionCustodied'>
	mail: {
		status(connectionId: string): Promise<MailAccountStatusV1>
		bind(input: {
			connectionId: string
			purpose: MailCredentialPurposeV1
			expectedBindingRevision: bigint
			credentialRevision: bigint
		}): Promise<MailCredentialBindingReceiptV1>
	}
	oauth: Pick<MailGmailOAuthClientV1, 'start' | 'complete'>
}

export type MailLegacyRecoveryResultV1 =
	| {
		kind: 'gmail'
		state: 'reauthorization_required'
		oauth: {
			operationId: string
			connectionId: string
			started: GmailOAuthStartedV1
		}
	}
	| {
		kind: 'icloud'
		state: 'ready' | 'applied_pending_readiness'
	}

export class MailLegacyRecoveryWorkflowV1 {
	constructor(private readonly ports: MailLegacyRecoveryPortsV1 = defaultPorts()) {}

	async recover(input: {
		registrationId: string
		plan: LegacyProviderRecoveryPlanV1
		candidate: LegacyProviderRecoveryCandidateV1
	}): Promise<MailLegacyRecoveryResultV1> {
		if (input.candidate.kind !== 'gmail' && input.candidate.kind !== 'icloud') {
			throw new Error('mail legacy recovery candidate is invalid')
		}
		const source = await this.ports.source.source(
			input.plan.recoverySessionId,
			input.candidate.sourceHandle,
		)
		if (source.kind !== input.candidate.kind
			|| source.sourceHandle !== input.candidate.sourceHandle) {
			throw new Error('mail legacy recovery source is invalid')
		}
		const operation = (step: string) => legacyRecoveryOperationIdV1(
			input.plan.bundleFingerprintSha256,
			input.candidate.sourceHandle,
			`mail_${source.kind}_${step}`,
		)
		const target = await this.ports.configuration.createTarget(
			input.registrationId,
			await operation('create_target'),
		)
		let settingsRevision = target.desiredRevision
		if (target.applyState === 'blocked_config') {
			const applied = await this.ports.configuration.apply({
				registrationId: input.registrationId,
				expectedDesiredRevision: target.desiredRevision,
				storageCapabilityId: MAIL_STORAGE_CAPABILITY_ID,
				configurationInstanceId: target.configurationInstanceId,
				requestHostBridge: false,
				values: source.kind === 'gmail'
					? mailGmailPreauthorizationSettings({
						connectionId: source.accountId,
						clientId: source.oauthClientId,
						redirectUri: source.oauthRedirectUri,
					})
					: mailImapSettings({
						registrationId: input.registrationId,
						expectedDesiredRevision: target.desiredRevision,
						connectionId: source.accountId,
						imapHost: source.imapHost,
						imapPort: BigInt(source.imapPort),
						username: source.username,
					}),
				updateOperationId: await operation('update_settings'),
				applyOperationId: await operation('apply_settings'),
			})
			settingsRevision = applied.settings.desiredRevision
		} else if (target.applyState !== 'current') {
			throw new Error('mail legacy recovery settings outcome is ambiguous')
		}
		if (source.kind === 'gmail') {
			const operationId = await legacyRecoveryOperationKeyV1(
				input.plan.bundleFingerprintSha256,
				input.candidate.sourceHandle,
				`mail_gmail_oauth_revision_${settingsRevision}`,
			)
			const started = await this.ports.oauth.start(operationId, source.accountId)
			return {
				kind: 'gmail',
				state: 'reauthorization_required',
				oauth: { operationId, connectionId: source.accountId, started },
			}
		}

		const current = await this.ports.mail.status(source.accountId)
		const binding = current.binding.find(
			(entry) => entry.purpose
				=== MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
		)
		if (!binding?.credentialRevision) {
			const vault = await this.ports.vault.provisionCustodied({
				operationId: await operation('provision_imap_password'),
				targetRegistrationId: input.registrationId,
				capabilityId: 'mail.imap.credential-provisioning.v1',
				configurationInstanceId: target.configurationInstanceId,
				purposeId: 'mail_imap_password',
				secretClass: OwnerVaultSecretClassV1.PROVIDER_CREDENTIAL,
				action: OwnerVaultActionV1.CREATE,
				secretRevision: 1n,
			}, (authorized) => this.ports.source.sealSource({
				...authorized,
				recoverySessionId: input.plan.recoverySessionId,
				sourceHandle: input.candidate.sourceHandle,
				secretPurpose: 'icloud_imap_password',
			}))
			await this.ports.mail.bind({
				connectionId: source.accountId,
				purpose: MailCredentialPurposeV1.MAIL_CREDENTIAL_PURPOSE_IMAP_PASSWORD,
				expectedBindingRevision: 0n,
				credentialRevision: vault.secretRevision,
			})
		}
		const status = await this.ports.mail.status(source.accountId)
		return {
			kind: 'icloud',
			state: status.readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
				? 'ready'
				: 'applied_pending_readiness',
		}
	}

	async completeGmail(
		result: Extract<MailLegacyRecoveryResultV1, { kind: 'gmail' }>,
		input: { returnedState: string; authorizationCode: string },
	): Promise<MailAcceptedV1> {
		return this.ports.oauth.complete({
			operationId: result.oauth.operationId,
			connectionId: result.oauth.connectionId,
			setupId: result.oauth.started.setupId,
			state: required(input.returnedState),
			authorizationCode: required(input.authorizationCode),
		})
	}
}

function defaultPorts(): MailLegacyRecoveryPortsV1 {
	return {
		source: createLegacyProviderRecoveryHostV1(),
		configuration: new ManagedIntegrationSetupV1(),
		vault: new OwnerVaultProvisioningClientV1(),
		mail: { status: getMailAccountStatus, bind: bindMailCredential },
		oauth: new MailGmailOAuthClientV1(),
	}
}

function required(value: string): string {
	const normalized = value.trim()
	if (!normalized) throw new Error('Gmail OAuth completion input is required')
	return normalized
}
