import {
	addMessageLabel,
	exportMessage,
	extractMessageNotes,
	extractMessageTasks,
	generateAiReply,
	snoozeMessage,
	toggleMessageImportant,
	toggleMessageMute,
	toggleMessagePin,
	translateMessage,
	type MailMessageInsight,
	type MessageExportResponse
} from '$lib/api';

export async function handleTogglePin(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		const result = await toggleMessagePin(messageId);
		return { success: true, message: result.pinned ? 'Message pinned' : 'Message unpinned' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Pin action failed' };
	}
}

export async function handleToggleImportant(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		const result = await toggleMessageImportant(messageId);
		return {
			success: true,
			message: result.important ? 'Marked important' : 'Removed important'
		};
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Important action failed' };
	}
}

export async function handleToggleMute(messageId: string): Promise<{ success: boolean; message: string }> {
	try {
		const result = await toggleMessageMute(messageId);
		return { success: true, message: result.pinned ? 'Message muted' : 'Message unmuted' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Mute action failed' };
	}
}

export async function handleSnoozeMessage(messageId: string, hours = 24): Promise<{ success: boolean; message: string }> {
	const until = new Date(Date.now() + hours * 60 * 60 * 1000).toISOString();
	try {
		await snoozeMessage(messageId, until);
		return { success: true, message: 'Message snoozed' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Snooze action failed' };
	}
}

export async function handleAddMessageLabel(messageId: string, label: string): Promise<{ success: boolean; message: string }> {
	const trimmed = label.trim();
	if (!trimmed) return { success: false, message: 'Label is required' };
	try {
		await addMessageLabel(messageId, trimmed);
		return { success: true, message: 'Label added' };
	} catch (error) {
		return { success: false, message: error instanceof Error ? error.message : 'Label action failed' };
	}
}

export async function handleExportMessage(
	messageId: string,
	format: 'md' | 'eml' | 'json'
): Promise<{ success: boolean; error: string; result: MessageExportResponse | null }> {
	try {
		const result = await exportMessage(messageId, format);
		return { success: true, error: '', result };
	} catch (error) {
		return {
			success: false,
			error: error instanceof Error ? error.message : 'Message export failed',
			result: null
		};
	}
}

export function safeMessageExportFilename(filename: string | null | undefined): string {
	const fallback = 'message-export.eml';
	const candidate = filename?.trim() || fallback;
	return candidate.replace(/[<>:"/\\|?*\u0000-\u001f]/g, '_').slice(0, 180) || fallback;
}

export function downloadMessageExport(exported: MessageExportResponse): boolean {
	if (typeof document === 'undefined' || typeof URL === 'undefined') {
		return false;
	}

	const blob = new Blob([exported.content], { type: exported.content_type || 'application/octet-stream' });
	const url = URL.createObjectURL(blob);
	const link = document.createElement('a');
	link.href = url;
	link.download = safeMessageExportFilename(exported.filename);
	link.style.display = 'none';
	document.body.appendChild(link);
	link.click();
	link.remove();
	URL.revokeObjectURL(url);
	return true;
}

export async function handleGenerateAiReply(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const aiReply = await generateAiReply(messageId, { tone: 'professional' });
		return { success: true, message: 'AI reply generated', insightPatch: { aiReply } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'AI reply generation failed',
			insightPatch: {}
		};
	}
}

export async function handleExtractTasks(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const response = await extractMessageTasks(messageId);
		return { success: true, message: 'Tasks extracted', insightPatch: { tasks: response.tasks } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Task extraction failed',
			insightPatch: {}
		};
	}
}

export async function handleExtractNotes(messageId: string): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const response = await extractMessageNotes(messageId);
		return { success: true, message: 'Notes extracted', insightPatch: { notes: response.notes } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Note extraction failed',
			insightPatch: {}
		};
	}
}

export async function handleTranslateMessage(messageId: string, targetLanguage = 'en'): Promise<{
	success: boolean;
	message: string;
	insightPatch: Partial<MailMessageInsight>;
}> {
	try {
		const translation = await translateMessage(messageId, targetLanguage);
		return { success: true, message: 'Translation requested', insightPatch: { translation } };
	} catch (error) {
		return {
			success: false,
			message: error instanceof Error ? error.message : 'Translation failed',
			insightPatch: {}
		};
	}
}

export async function askAiAboutSelectedMessage(
	message: { message_id: string; subject: string } | null
): Promise<{ aiQuestion: string } | null> {
	if (!message) return null;
	return {
		aiQuestion: `Answer from local sources for message ${message.message_id}: ${message.subject}`
	};
}
