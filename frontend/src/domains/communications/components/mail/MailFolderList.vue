<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '@/platform/i18n'
import { Badge, Icon } from '@/shared/ui'
import '../communicationDomainElements.css'
import type { MailFolderModel, MailFolderRow } from './mailFolders'
import { mailFolderExpandableIds, mailFolderExpandedIds, mailFolderPresentation, mailFolderRows } from './mailFolders'

const props = withDefaults(defineProps<{
  folders: readonly MailFolderModel[]
  activeFolderId?: string
}>(), {
  activeFolderId: ''
})

const emit = defineEmits<{
  select: [folder: MailFolderModel]
}>()

const { t } = useI18n()
const expandedFolderIds = ref<readonly string[]>(mailFolderExpandableIds(props.folders))
const allFolderRows = computed(() => mailFolderRows(props.folders))
const visibleFolderRows = computed(() => mailFolderRows(props.folders, expandedFolderIds.value))

watch(
  () => props.folders,
  (folders) => {
    expandedFolderIds.value = mailFolderExpandableIds(folders)
  }
)

function folderIsActive(folder: MailFolderModel): boolean {
  return props.activeFolderId ? props.activeFolderId === folder.id : Boolean(folder.selected)
}

function selectFolder(folder: MailFolderModel): void {
  emit('select', folder)
}

function setFolderExpanded(row: MailFolderRow, expanded: boolean): void {
  if (!row.hasChildren) return
  expandedFolderIds.value = mailFolderExpandedIds(expandedFolderIds.value, row.folder.id, expanded)
}

function toggleFolder(row: MailFolderRow): void {
  setFolderExpanded(row, !row.expanded)
}

function handleFolderKeydown(row: MailFolderRow, event: KeyboardEvent): void {
  if (event.key === 'ArrowRight' && row.hasChildren && !row.expanded) {
    event.preventDefault()
    setFolderExpanded(row, true)
  }
  if (event.key === 'ArrowLeft' && row.hasChildren && row.expanded) {
    event.preventDefault()
    setFolderExpanded(row, false)
  }
}

function folderDepthClass(row: MailFolderRow): string {
  return `mail-folder-list__item--depth-${Math.min(row.depth, 4)}`
}

function folderToggleAriaLabel(row: MailFolderRow): string {
  const action = row.expanded ? t('Collapse folder') : t('Expand folder')
  return `${action}: ${t(row.folder.label)}`
}

function folderAriaLabel(folder: MailFolderModel): string {
  const parts = [t(folder.label)]
  if (folder.unreadCount) parts.push(t('{count} unread', { count: folder.unreadCount }))
  if (typeof folder.count === 'number') parts.push(t('{count} total', { count: folder.count }))
  return parts.join(', ')
}
</script>

<template>
	<nav class="mail-folder-list" :aria-label="t('Mail folders')">
		<header class="mail-folder-list__header">
			<span>{{ t('Folders') }}</span>
			<Badge variant="neutral">{{ allFolderRows.length }}</Badge>
		</header>
		<div class="mail-folder-list__items" role="tree">
			<div
				v-for="row in visibleFolderRows"
				:key="row.folder.id"
				:class="[
					'mail-folder-list__item',
					folderDepthClass(row),
					row.hasChildren && 'mail-folder-list__item--parent',
					folderIsActive(row.folder) && 'mail-folder-list__item--active'
				]"
				role="treeitem"
				:aria-current="folderIsActive(row.folder) ? 'page' : undefined"
				:aria-expanded="row.hasChildren ? row.expanded : undefined"
				:aria-label="folderAriaLabel(row.folder)"
				:aria-level="row.depth"
			>
				<button
					v-if="row.hasChildren"
					type="button"
					class="mail-folder-list__toggle"
					:aria-label="folderToggleAriaLabel(row)"
					:aria-expanded="row.expanded"
					@click.stop="toggleFolder(row)"
				>
					<Icon :icon="row.expanded ? 'tabler:chevron-down' : 'tabler:chevron-right'" size="0.875rem" aria-hidden="true" />
				</button>
				<span v-else class="mail-folder-list__toggle-placeholder" aria-hidden="true"></span>
				<button
					type="button"
					class="mail-folder-list__select"
					:aria-label="folderAriaLabel(row.folder)"
					@click="selectFolder(row.folder)"
					@keydown="handleFolderKeydown(row, $event)"
				>
					<span class="mail-folder-list__depth-line" aria-hidden="true">
						<Icon v-if="row.depth > 1" icon="tabler:corner-down-right" size="0.75rem" />
					</span>
					<span
						:class="[
							'mail-folder-list__icon',
							`mail-folder-list__icon--${mailFolderPresentation(row.folder).tone}`
						]"
						aria-hidden="true"
					>
						<Icon :icon="mailFolderPresentation(row.folder).icon" size="1rem" />
					</span>
					<span class="mail-folder-list__label">{{ t(row.folder.label) }}</span>
					<span class="mail-folder-list__metrics">
						<span v-if="row.folder.unreadCount" class="mail-folder-list__unread">{{ row.folder.unreadCount }}</span>
						<span v-if="typeof row.folder.count === 'number'" class="mail-folder-list__count">{{ row.folder.count }}</span>
					</span>
				</button>
			</div>
		</div>
	</nav>
</template>
