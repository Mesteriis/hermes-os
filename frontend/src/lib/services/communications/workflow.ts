import {
	runWorkflowAction,
	type CommunicationMessageSummary,
	type WorkflowActionKind,
	type WorkflowActionRequest,
	type WorkflowActionResponse
} from '$lib/api';

export async function handleWorkflowActionRequest(
	request: WorkflowActionRequest
): Promise<{ success: boolean; message: string; result: WorkflowActionResponse | null }> {
	try {
		const result = await runWorkflowAction(request);
		return { success: true, message: workflowActionStatusLabel(result), result };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Workflow action failed',
			result: null
		};
	}
}

export function buildWorkflowActionRequest(
	action: WorkflowActionKind,
	message: Pick<CommunicationMessageSummary, 'message_id' | 'subject'> | null,
	commandId = workflowCommandId(action)
): WorkflowActionRequest {
	const request: WorkflowActionRequest = {
		command_id: commandId,
		action
	};
	if (message) {
		request.source = { kind: 'communication_message', id: message.message_id };
	}
	if (['create_task', 'create_note', 'create_document', 'link_document', 'create_event'].includes(action)) {
		request.input = { title: message?.subject?.trim() || 'Untitled' };
	}
	return request;
}

function workflowCommandId(action: WorkflowActionKind): string {
	const entropy =
		typeof crypto !== 'undefined' && 'randomUUID' in crypto
			? crypto.randomUUID()
			: `${Date.now()}-${Math.random().toString(16).slice(2)}`;
	return `mail-${action}-${entropy}`;
}

function workflowActionStatusLabel(result: WorkflowActionResponse): string {
	switch (result.action) {
		case 'create_task':
			return 'Task created';
		case 'create_note':
			return 'Note created';
		case 'create_document':
		case 'link_document':
			return 'Document linked';
		case 'create_event':
			return 'Event created';
		case 'create_contact':
			return 'Contact created';
		case 'archive':
			return 'Archived';
		case 'reply':
			return 'Reply opened';
		default:
			return 'Workflow action completed';
	}
}
