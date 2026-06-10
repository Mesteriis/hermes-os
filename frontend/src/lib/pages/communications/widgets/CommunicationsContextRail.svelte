<script lang="ts">
	import Icon from '@iconify/svelte';
	import WidgetEditChrome from '$lib/components/shared/WidgetEditChrome.svelte';

	interface Props {
		selectedCommunication: unknown | null;
		projects: unknown[];
		tasks: unknown[];
		isLayoutEditing: boolean;
		isWidgetVisible: (id: string) => boolean;
		senderLabel: (sender: string) => string;
		senderEmail: (sender: string) => string;
		communicationChannelLabel: (kind: string) => string;
		messageTime: (msg: unknown) => string;
	}

	let {
		selectedCommunication,
		projects,
		tasks,
		isLayoutEditing,
		isWidgetVisible,
		senderLabel,
		senderEmail,
		communicationChannelLabel,
		messageTime
	}: Props = $props();
</script>

<aside class="context-rail">
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-sender-profile" data-widget-hidden={!isWidgetVisible('communications-sender-profile')}>
		<WidgetEditChrome widgetId="communications-sender-profile" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel profile-panel">
			<div class="profile-head"><img src="/assets/hermes-reference-avatar.png" alt="" /><div><h2>{selectedCommunication ? senderLabel((selectedCommunication as Record<string, unknown>).sender as string) : 'No sender selected'}</h2><p>{selectedCommunication ? communicationChannelLabel((selectedCommunication as Record<string, unknown>).channel_kind as string) : 'No channel'}</p><small>{selectedCommunication ? senderEmail((selectedCommunication as Record<string, unknown>).sender as string) : 'No local message selected'}</small></div></div>
			<div class="quick-icons">
				<button type="button" disabled><Icon icon="tabler:mail" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:phone" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:brand-telegram" width="17" height="17" /></button>
				<button type="button" disabled><Icon icon="tabler:brand-whatsapp" width="17" height="17" /></button>
			</div>
		</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-summary" data-widget-hidden={!isWidgetVisible('communications-summary')}>
		<WidgetEditChrome widgetId="communications-summary" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Summary</h2><p>{selectedCommunication ? `Stored from ${(selectedCommunication as Record<string, unknown>).account_id as string}. Channel ${communicationChannelLabel((selectedCommunication as Record<string, unknown>).channel_kind as string)}. Provider record ${(selectedCommunication as Record<string, unknown>).provider_record_id as string}.` : 'Local communication metadata will appear after messages are imported.'}</p><button type="button" class="link-row" disabled>View full profile <Icon icon="tabler:arrow-right" width="15" height="15" /></button></section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-message-metadata" data-widget-hidden={!isWidgetVisible('communications-message-metadata')}>
		<WidgetEditChrome widgetId="communications-message-metadata" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Message Metadata</h2>{#if selectedCommunication}<ul class="detail-list"><li><Icon icon="tabler:users" width="17" height="17" /> {((selectedCommunication as Record<string, unknown>).recipients as unknown[]).length} recipients</li><li><Icon icon="tabler:paperclip" width="17" height="17" /> {(selectedCommunication as Record<string, unknown>).attachment_count as number} attachments</li><li><Icon icon="tabler:clock" width="17" height="17" /> {messageTime(selectedCommunication)}</li></ul>{:else}<p>No message selected.</p>{/if}</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-related-projects" data-widget-hidden={!isWidgetVisible('communications-related-projects')}>
		<WidgetEditChrome widgetId="communications-related-projects" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Related Projects</h2>{#each (projects as unknown[]).slice(0, 2) as project}<div class="related-row"><span class="round-icon {(project as Record<string, unknown>).tone as string}"><Icon icon={(project as Record<string, unknown>).icon as string} width="16" height="16" /></span><strong>{(project as Record<string, unknown>).name as string}</strong><em>{(project as Record<string, unknown>).progress as number}%</em></div>{/each}</section>
	</div>
	<div class="widget-frame" class:editing={isLayoutEditing} data-widget-id="communications-active-tasks" data-widget-hidden={!isWidgetVisible('communications-active-tasks')}>
		<WidgetEditChrome widgetId="communications-active-tasks" {isLayoutEditing} isSelected={false} onConfigure={() => {}} />
		<section class="panel info-card"><h2>Active Tasks</h2>{#each (tasks as unknown[]).slice(0, 3) as task}<label class="mini-check"><input type="checkbox" />{(task as Record<string, unknown>).title as string}<em>{((task as Record<string, unknown>).due as string).split(' ')[0]}</em></label>{/each}</section>
	</div>
</aside>
