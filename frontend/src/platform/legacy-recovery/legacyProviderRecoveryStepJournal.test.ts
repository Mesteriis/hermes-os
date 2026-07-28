import { describe, expect, it, vi } from 'vitest'

import {
	LegacyProviderRecoveryOutcomeUnknownErrorV1,
} from './legacyProviderRecoveryHost'
import { LegacyProviderRecoveryStepJournalV1 } from './legacyProviderRecoveryStepJournal'

describe('LegacyProviderRecoveryStepJournalV1', () => {
	it('stops before replay when the persisted step outcome is unknown', async () => {
		const beginStep = vi.fn().mockResolvedValue({
			disposition: 'outcome_unknown',
			operationId: new Uint8Array(16).fill(7),
		})
		const host = {
			beginStep,
			completeStep: vi.fn(),
			finishCandidate: vi.fn(),
		}
		const journal = new LegacyProviderRecoveryStepJournalV1(
			host as never,
			'a'.repeat(32),
			'b'.repeat(64),
			false,
		)

		await expect(journal.begin('mail_icloud_create_target')).rejects.toBeInstanceOf(
			LegacyProviderRecoveryOutcomeUnknownErrorV1,
		)
		expect(host.completeStep).not.toHaveBeenCalled()
		expect(beginStep).toHaveBeenCalledWith(expect.objectContaining({
			explicitRetry: false,
		}))
	})

	it('sends explicit retry admission only after the owner chooses retry', async () => {
		const beginStep = vi.fn().mockResolvedValue({
			disposition: 'execute',
			operationId: new Uint8Array(16).fill(9),
		})
		const journal = new LegacyProviderRecoveryStepJournalV1(
			{ beginStep } as never,
			'a'.repeat(32),
			'b'.repeat(64),
			true,
		)

		await journal.begin('telegram_provision_account', 'telegram-registration')

		expect(beginStep).toHaveBeenCalledWith(expect.objectContaining({
			explicitRetry: true,
			targetConfigurationInstanceId: 'telegram-registration',
		}))
	})
})
