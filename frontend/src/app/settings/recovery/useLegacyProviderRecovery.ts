import { computed, onBeforeUnmount, ref, shallowRef } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/hermes/gateway/v1/client_bootstrap_pb'
import { MailLegacyRecoveryWorkflowV1 } from '../../../integrations/mail/recovery/mailLegacyRecoveryWorkflow'
import type { MailLegacyRecoveryResultV1 } from '../../../integrations/mail/recovery/mailLegacyRecoveryWorkflow'
import {
	TelegramLegacyRecoveryWorkflowV1,
	type TelegramLegacyRecoveryResultV1,
} from '../../../integrations/telegram/recovery/telegramLegacyRecoveryWorkflow'
import {
	createLegacyProviderRecoveryHostV1,
	hasLegacyProviderRecoveryHostV1,
	type LegacyProviderRecoveryCandidateV1,
	type LegacyProviderRecoveryPlanV1,
} from '../../../platform/legacy-recovery'
import { fetchClientBootstrap } from '../../../platform/gateway/clientBootstrap'
import type {
	LegacyProviderCandidateProgressV1,
} from './legacyProviderRecoveryPresentation'

export function useLegacyProviderRecovery(
	mailModule: () => ClientModuleBootstrapV1 | null,
	telegramModule: () => ClientModuleBootstrapV1 | null,
	source = createLegacyProviderRecoveryHostV1(),
	mail = new MailLegacyRecoveryWorkflowV1(),
	telegram = new TelegramLegacyRecoveryWorkflowV1(),
) {
	const plan = shallowRef<LegacyProviderRecoveryPlanV1>()
	const progress = ref<Record<string, LegacyProviderCandidateProgressV1>>({})
	const mailResults = shallowRef<MailLegacyRecoveryResultV1[]>([])
	const telegramResult = shallowRef<TelegramLegacyRecoveryResultV1>()
	const returnedState = ref('')
	const authorizationCode = ref('')
	const oauthAccepted = ref(false)
	const busy = ref(false)
	const message = ref('')
	const messageTone = ref<'neutral' | 'success' | 'error'>('neutral')
	const available = hasLegacyProviderRecoveryHostV1()
	let sessionActive = false
	const canInspect = computed(() => available && !busy.value)
	const canRecover = computed(() => Boolean(
		plan.value
		&& mailModule()?.settings
		&& telegramModule()?.settings
		&& !busy.value,
	))
	const gmailResult = computed(() =>
		mailResults.value.find((result) => result.kind === 'gmail'))

	async function inspect(): Promise<void> {
		if (!canInspect.value) return
		await run(async () => {
			await cancelActiveSession()
			const next = await source.start()
			plan.value = next
			sessionActive = true
			progress.value = Object.fromEntries(
				next.candidates.map((candidate) => [candidate.sourceHandle, 'pending']),
			)
			mailResults.value = []
			telegramResult.value = undefined
			oauthAccepted.value = false
			message.value = 'Recovery bundle verified. Three active provider accounts are ready for owner-authorized recovery.'
		}, 'The private recovery bundle could not be verified.')
	}

	async function recoverAll(): Promise<void> {
		const currentPlan = plan.value
		const currentMail = mailModule()
		const currentTelegram = telegramModule()
		if (!currentPlan || !currentMail?.settings || !currentTelegram?.settings) return
		await run(async () => {
			for (const candidate of orderedCandidates(currentPlan.candidates)) {
				progress.value = { ...progress.value, [candidate.sourceHandle]: 'running' }
				try {
					if (candidate.kind === 'telegram_user') {
						const latest = await fetchClientBootstrap()
						const latestTelegram = latest.modules.find(
							(module) => module.registrationId === currentTelegram.registrationId,
						)
						if (!latestTelegram?.settings) {
							throw new Error('Telegram Settings state is unavailable')
						}
						telegramResult.value = await telegram.recover({
							registrationId: currentTelegram.registrationId,
							expectedDesiredRevision: latestTelegram.settings.desiredRevision,
							plan: currentPlan,
							candidate,
						})
					} else {
						mailResults.value = [
							...mailResults.value.filter((result) => result.kind !== candidate.kind),
							await mail.recover({
								registrationId: currentMail.registrationId,
								plan: currentPlan,
								candidate,
							}),
						]
					}
					progress.value = { ...progress.value, [candidate.sourceHandle]: 'completed' }
				} catch (error) {
					progress.value = { ...progress.value, [candidate.sourceHandle]: 'failed' }
					throw error
				}
			}
			await cancelActiveSession()
			message.value = 'Two Mail targets and one Telegram user target were recovered through their provider-owned contracts.'
		}, 'Recovery stopped at the current provider step. Completed idempotent steps are safe to retry.')
	}

	async function completeGmail(): Promise<void> {
		const current = gmailResult.value
		if (!current || current.kind !== 'gmail') return
		await run(async () => {
			await mail.completeGmail(current, {
				returnedState: returnedState.value,
				authorizationCode: authorizationCode.value,
			})
			oauthAccepted.value = true
			message.value = 'Gmail OAuth completion was accepted. Mail readiness will update after reconciliation.'
		}, 'Gmail OAuth completion was rejected.')
		authorizationCode.value = ''
	}

	async function run(action: () => Promise<void>, failure: string): Promise<void> {
		busy.value = true
		message.value = ''
		messageTone.value = 'neutral'
		try {
			await action()
			messageTone.value = 'success'
		} catch (error) {
			message.value = `${failure} (${safeFailureCode(error)})`
			messageTone.value = 'error'
		} finally {
			busy.value = false
		}
	}

	async function cancelActiveSession(): Promise<void> {
		const current = plan.value
		if (!current || !sessionActive) return
		sessionActive = false
		await source.cancel(current.recoverySessionId).catch(() => undefined)
	}

	onBeforeUnmount(() => {
		void cancelActiveSession()
		authorizationCode.value = ''
		returnedState.value = ''
	})

	return {
		plan,
		progress,
		mailResults,
		telegramResult,
		returnedState,
		authorizationCode,
		oauthAccepted,
		busy,
		message,
		messageTone,
		available,
		canInspect,
		canRecover,
		gmailResult,
		inspect,
		recoverAll,
		completeGmail,
	}
}

function safeFailureCode(error: unknown): string {
	if (!(error instanceof Error)) return 'unexpected_error'
	if (error.message === 'legacy provider recovery host response is invalid') {
		return 'invalid_host_response'
	}
	if (/^legacy provider recovery host rejected request \([0-9]{3}\)$/.test(error.message)) {
		return 'host_rejected_request'
	}
	if (error.message === 'legacy provider recovery host is unavailable') {
		return 'host_unavailable'
	}
	if (error instanceof TypeError) {
		if (/source\.start|start is not a function/.test(error.message)) {
			return 'source_start_unavailable'
		}
		if (/Object\.fromEntries|fromEntries/.test(error.message)) {
			return 'progress_initialization_failed'
		}
		if (/fetch|json|Response/.test(error.message)) {
			if (/Unexpected.*JSON|JSON.*position|JSON input/.test(error.message)) {
				return 'response_json_invalid'
			}
			if (/undefined.*json|json.*undefined/.test(error.message)) {
				return 'response_missing'
			}
			if (/Response/.test(error.message)) {
				return 'response_api_failed'
			}
			if (/fetch/.test(error.message) && !/fetchImpl/.test(error.message)) {
				return 'fetch_api_failed'
			}
			if (/response\.json is not a function/.test(error.message)) {
				return 'response_json_unavailable'
			}
			if (/fetchImpl|is not a function/.test(error.message)) {
				return 'fetch_adapter_unavailable'
			}
			if (/Failed to fetch|Load failed|NetworkError/.test(error.message)) {
				return 'browser_network_failed'
			}
			if (/body|stream|unusable|already read/i.test(error.message)) {
				return 'response_body_unavailable'
			}
			return 'browser_transport_failed'
		}
		if (/map|candidates/.test(error.message)) {
			return 'candidate_projection_failed'
		}
		return 'client_type_error'
	}
	return 'operation_rejected'
}

function orderedCandidates(
	candidates: readonly LegacyProviderRecoveryCandidateV1[],
): LegacyProviderRecoveryCandidateV1[] {
	const order = { icloud: 0, gmail: 1, telegram_user: 2 } as const
	return [...candidates].sort((left, right) => order[left.kind] - order[right.kind])
}
