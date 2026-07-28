import {
	type LegacyProviderRecoveryHostV1,
	LegacyProviderRecoveryOutcomeUnknownErrorV1,
	type LegacyProviderRecoveryStepIdentifierV1,
	type LegacyProviderRecoveryStepV1,
	type LegacyProviderRecoveryTerminalStateV1,
} from './legacyProviderRecoveryHost'

export class LegacyProviderRecoveryStepJournalV1 {
	constructor(
		private readonly host: LegacyProviderRecoveryHostV1,
		private readonly recoverySessionId: string,
		private readonly sourceHandle: string,
		private readonly explicitRetry: boolean,
	) {}

	async begin(
		stepIdentifier: LegacyProviderRecoveryStepIdentifierV1,
		targetConfigurationInstanceId?: string,
	): Promise<LegacyProviderRecoveryStepV1> {
		const step = await this.inspect(stepIdentifier, targetConfigurationInstanceId)
		if (step.disposition === 'outcome_unknown') {
			throw new LegacyProviderRecoveryOutcomeUnknownErrorV1()
		}
		return step
	}

	async inspect(
		stepIdentifier: LegacyProviderRecoveryStepIdentifierV1,
		targetConfigurationInstanceId?: string,
	): Promise<LegacyProviderRecoveryStepV1> {
		return this.host.beginStep({
			recoverySessionId: this.recoverySessionId,
			sourceHandle: this.sourceHandle,
			stepIdentifier,
			targetConfigurationInstanceId,
			explicitRetry: this.explicitRetry,
		})
	}

	async complete(
		stepIdentifier: LegacyProviderRecoveryStepIdentifierV1,
		step: LegacyProviderRecoveryStepV1,
		input: {
			targetConfigurationInstanceId?: string
			publicRevision?: bigint
		} = {},
	): Promise<void> {
		await this.host.completeStep({
			recoverySessionId: this.recoverySessionId,
			sourceHandle: this.sourceHandle,
			stepIdentifier,
			operationId: step.operationId,
			targetConfigurationInstanceId: input.targetConfigurationInstanceId,
			publicRevision: input.publicRevision,
		})
	}

	async finish(
		targetConfigurationInstanceId: string,
		terminalState: LegacyProviderRecoveryTerminalStateV1,
	): Promise<void> {
		await this.host.finishCandidate({
			recoverySessionId: this.recoverySessionId,
			sourceHandle: this.sourceHandle,
			targetConfigurationInstanceId,
			terminalState,
		})
	}
}

export function legacyProviderRecoveryOperationKeyFromStepV1(
	step: LegacyProviderRecoveryStepV1,
): string {
	return `legacy-recovery-${Array.from(step.operationId, (byte) =>
		byte.toString(16).padStart(2, '0')).join('')}`
}
