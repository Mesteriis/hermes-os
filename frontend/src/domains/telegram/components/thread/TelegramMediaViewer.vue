<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../platform/i18n'
import Icon from '../../../../shared/ui/Icon.vue'
import type { TelegramAttachmentHint } from '../../types/telegram'

const { t } = useI18n()

const props = defineProps<{
  attachment: TelegramAttachmentHint | null
}>()

const emit = defineEmits<{
  close: []
}>()

const previewKind = computed(() => {
  const kind = props.attachment?.kind
  if (kind === 'photo') return 'image'
  if (kind === 'video') return 'video'
  if (kind === 'audio' || kind === 'voice') return 'audio'
  return 'file'
})

function formatBytes(bytes: number | null): string {
  if (bytes == null) return t('Unknown size')
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <div v-if="attachment" class="telegram-media-viewer-backdrop" @click.self="emit('close')">
    <section class="telegram-media-viewer">
      <header class="telegram-media-viewer__header">
        <div>
          <h3>{{ attachment.fileName }}</h3>
          <p>{{ attachment.mimeType ?? attachment.kind }} · {{ formatBytes(attachment.sizeBytes) }}</p>
        </div>
        <button type="button" :title="t('Close')" @click="emit('close')">
          <Icon icon="tabler:x" width="18" height="18" />
        </button>
      </header>

      <div class="telegram-media-viewer__body">
        <img
          v-if="previewKind === 'image' && attachment.localPath"
          :src="attachment.localPath"
          :alt="attachment.fileName"
        />
        <video
          v-else-if="previewKind === 'video' && attachment.localPath"
          :src="attachment.localPath"
          controls
        ></video>
        <audio
          v-else-if="previewKind === 'audio' && attachment.localPath"
          :src="attachment.localPath"
          controls
        ></audio>
        <div v-else class="telegram-media-viewer__empty">
          <Icon icon="tabler:file-search" width="28" height="28" />
          <p>{{ t('Preview is available after the Telegram media file is downloaded locally.') }}</p>
          <small>{{ t('Current state:') }} {{ attachment.downloadState }}</small>
        </div>
      </div>

      <footer class="telegram-media-viewer__footer">
        <span>{{ t('Attachment ID') }}: {{ attachment.providerAttachmentId || t('Unavailable') }}</span>
        <span>{{ t('Message ID') }}: {{ attachment.messageId }}</span>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.telegram-media-viewer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(20, 24, 28, 0.58);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  z-index: 50;
}
.telegram-media-viewer {
  width: min(880px, 100%);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  background: var(--color-surface, #fff);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 22px 64px rgba(0, 0, 0, 0.24);
}
.telegram-media-viewer__header,
.telegram-media-viewer__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 18px;
  border-bottom: 1px solid var(--color-border, #e0e0e0);
}
.telegram-media-viewer__footer {
  border-top: 1px solid var(--color-border, #e0e0e0);
  border-bottom: none;
  font-size: 11px;
  color: var(--color-text-secondary, #777);
  flex-wrap: wrap;
}
.telegram-media-viewer__header h3,
.telegram-media-viewer__header p {
  margin: 0;
}
.telegram-media-viewer__header h3 {
  font-size: 14px;
  color: var(--color-text, #333);
}
.telegram-media-viewer__header p {
  font-size: 11px;
  color: var(--color-text-secondary, #777);
}
.telegram-media-viewer__header button {
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--color-text-secondary, #777);
  padding: 4px;
  border-radius: 6px;
}
.telegram-media-viewer__body {
  padding: 18px;
  overflow: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 320px;
  background:
    radial-gradient(circle at top left, rgba(214, 234, 248, 0.9), transparent 38%),
    linear-gradient(180deg, #f7fafc 0%, #eef3f8 100%);
}
.telegram-media-viewer__body img,
.telegram-media-viewer__body video {
  max-width: 100%;
  max-height: 62vh;
  border-radius: 12px;
  object-fit: contain;
  box-shadow: 0 12px 34px rgba(37, 61, 84, 0.18);
}
.telegram-media-viewer__body audio {
  width: min(520px, 100%);
}
.telegram-media-viewer__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
  color: var(--color-text-secondary, #667085);
}
.telegram-media-viewer__empty p,
.telegram-media-viewer__empty small {
  margin: 0;
}
</style>
