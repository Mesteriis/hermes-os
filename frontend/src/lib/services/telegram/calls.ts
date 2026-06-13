import {
	fetchCallTranscript,
	saveCallTranscriptFixture,
	saveTelegramCall,
	type CallTranscript
} from '$lib/api';
import { parseJsonObject } from './parsing';

export async function saveTelegramCallFixture(params: {
	call_id: string;
	account_id: string;
	provider_call_id: string;
	provider_chat_id: string;
	direction: 'incoming' | 'outgoing';
	call_state: 'ringing' | 'active' | 'ended' | 'missed' | 'declined' | 'failed';
	started_at: string;
	ended_at: string;
	transcription_policy_id: string;
	metadata_text: string;
}): Promise<{
	message: string;
	error: string;
	callId: string;
}> {
	try {
		const call = await saveTelegramCall({
			call_id: params.call_id,
			account_id: params.account_id,
			provider_call_id: params.provider_call_id,
			provider_chat_id: params.provider_chat_id,
			direction: params.direction,
			call_state: params.call_state,
			started_at: params.started_at.trim() || null,
			ended_at: params.ended_at.trim() || null,
			transcription_policy_id: params.transcription_policy_id.trim() || null,
			metadata: parseJsonObject(params.metadata_text, 'call metadata')
		});
		return {
			message: `Call ${call.call_id} saved`,
			error: '',
			callId: call.call_id
		};
	} catch (error) {
		return {
			message: '',
			error: error instanceof Error ? error.message : 'Telegram call save failed',
			callId: params.call_id
		};
	}
}

export async function saveCallTranscriptFixtureFromUi(params: {
	transcript_id: string;
	account_id: string;
	provider_chat_id: string;
	source_audio_ref: string;
	language_code: string;
	always_on_policy: boolean;
	selectedCallId: string;
}): Promise<{
	transcript: CallTranscript | null;
	message: string;
	error: string;
}> {
	if (!params.selectedCallId) {
		return { transcript: null, message: '', error: '' };
	}
	try {
		const transcript = await saveCallTranscriptFixture(
			params.selectedCallId,
			{
				transcript_id: params.transcript_id,
				account_id: params.account_id,
				provider_chat_id: params.provider_chat_id,
				source_audio_ref: params.source_audio_ref,
				language_code: params.language_code || undefined,
				always_on_policy: params.always_on_policy
			}
		);
		return {
			transcript,
			message: `Transcript ${transcript.transcript_id} saved`,
			error: ''
		};
	} catch (error) {
		return {
			transcript: null,
			message: '',
			error: error instanceof Error ? error.message : 'Call transcript save failed'
		};
	}
}

export async function loadSelectedCallTranscript(
	callId: string
): Promise<{
	transcript: CallTranscript | null;
	error: string;
}> {
	if (!callId) {
		return { transcript: null, error: '' };
	}
	try {
		const response = await fetchCallTranscript(callId);
		return { transcript: response.transcript, error: '' };
	} catch (error) {
		return {
			transcript: null,
			error: error instanceof Error ? error.message : 'Call transcript request failed'
		};
	}
}
