import {
	dryRunTelegramSend,
	saveAutomationPolicy,
	saveAutomationTemplate,
	type TelegramSendDryRunResponse
} from '$lib/api';
import { parseJsonObject, parseStringMap } from './parsing';

export async function saveTelegramAutomationTemplate(params: {
	template_id: string;
	name: string;
	body_template: string;
	required_variables_text: string;
}): Promise<{
	message: string;
	error: string;
	templateId: string;
}> {
	try {
		const template = await saveAutomationTemplate({
			template_id: params.template_id,
			name: params.name,
			body_template: params.body_template,
			required_variables: params.required_variables_text
				.split(',')
				.map((item) => item.trim())
				.filter(Boolean)
		});
		return {
			message: `Template ${template.template_id} saved`,
			error: '',
			templateId: template.template_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Automation template save failed',
			templateId: params.template_id
		};
	}
}

export async function saveTelegramAutomationPolicy(params: {
	policy_id: string;
	template_id: string;
	name: string;
	enabled: boolean;
	account_id: string;
	allowed_chat_ids_text: string;
	trigger_kind: string;
	max_sends_per_hour: number;
	quiet_hours_text: string;
	expires_at: string;
	conditions_text: string;
}): Promise<{
	message: string;
	error: string;
	policyId: string;
}> {
	try {
		const policy = await saveAutomationPolicy({
			policy_id: params.policy_id,
			template_id: params.template_id,
			name: params.name,
			enabled: params.enabled,
			account_id: params.account_id,
			allowed_chat_ids: params.allowed_chat_ids_text
				.split(',')
				.map((item) => item.trim())
				.filter(Boolean),
			trigger_kind: params.trigger_kind,
			max_sends_per_hour: Number(params.max_sends_per_hour),
			quiet_hours: parseJsonObject(params.quiet_hours_text, 'quiet hours'),
			expires_at: params.expires_at.trim() || null,
			conditions: parseJsonObject(params.conditions_text, 'conditions')
		});
		return {
			message: `Policy ${policy.policy_id} saved`,
			error: '',
			policyId: policy.policy_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Automation policy save failed',
			policyId: params.policy_id
		};
	}
}

export async function runTelegramAutomationDryRun(params: {
	policy_id: string;
	provider_chat_id: string;
	variables_text: string;
	source_context_text: string;
}): Promise<{
	result: TelegramSendDryRunResponse | null;
	message: string;
	error: string;
}> {
	try {
		const result = await dryRunTelegramSend({
			command_id: `telegram-dry-run-${crypto.randomUUID()}`,
			policy_id: params.policy_id,
			provider_chat_id: params.provider_chat_id,
			variables: parseStringMap(params.variables_text, 'variables'),
			source_context: parseJsonObject(params.source_context_text, 'source context')
		});
		return {
			result,
			message: `Dry-run accepted with preview hash ${result.rendered_preview_hash}`,
			error: ''
		};
	} catch (error) {
		return {
			result: null,
			message: '',
			error: error instanceof Error ? error.message : 'Telegram send dry-run failed'
		};
	}
}
