<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  icon: string
  size?: number | string
  class?: string
}>(), {
  size: 20,
})

type LocalIcon = readonly string[]

const generic: LocalIcon = ['M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0-18', 'M12 8v4l3 2']
const icons: Readonly<Record<string, LocalIcon>> = {
  'tabler:activity-heartbeat': ['M3 12h4l2-5l4 10l2-5h6'],
  'tabler:arrows-exchange': ['M7 7h11l-3-3', 'M18 7l-3 3', 'M17 17H6l3 3', 'M6 17l3-3'],
  'tabler:bell': ['M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9', 'M10 21h4'],
  'tabler:brand-telegram': ['M21 4L3 11l7 2l2 7l3-5l4 3z'],
  'tabler:brand-whatsapp': ['M20 11.5a8 8 0 0 1-11.7 7l-4.3 1l1.1-4.1A8 8 0 1 1 20 11.5', 'M9 9.5c.4 2 1.8 3.5 3.8 4l1.1-1.1l1.6.8c-.4 1-1.2 1.5-2.1 1.3c-3.1-.8-5.1-2.9-5.8-5.8c-.2-.9.3-1.7 1.3-2.1l.8 1.6z'],
  'tabler:calendar': ['M4 5h16v15H4z', 'M8 3v4', 'M16 3v4', 'M4 10h16'],
  'tabler:calendar-time': ['M4 5h16v15H4z', 'M8 3v4', 'M16 3v4', 'M4 10h16', 'M15 14v3l2 1'],
  'tabler:checkbox': ['M5 5h14v14H5z', 'M8 12l2.5 2.5L16 9'],
  'tabler:clipboard-check': ['M9 5h6', 'M9 3h6a2 2 0 0 1 2 2v1H7V5a2 2 0 0 1 2-2', 'M7 5H5v16h14V5h-2', 'M9 14l2 2l4-5'],
  'tabler:clock': ['M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0-18', 'M12 7v5l3 2'],
  'tabler:code': ['M8 9l-3 3l3 3', 'M16 9l3 3l-3 3', 'M14 5l-4 14'],
  'tabler:database': ['M5 6c0-2 14-2 14 0s-14 2-14 0', 'M5 6v6c0 2 14 2 14 0V6', 'M5 12v6c0 2 14 2 14 0v-6'],
  'tabler:database-cog': ['M5 6c0-2 10-2 10 0s-10 2-10 0', 'M5 6v6c0 2 10 2 10 0V6', 'M5 12v4c0 1 2 1.5 4 1.5', 'M18 16v4', 'M16.3 17l3.4 2', 'M19.7 17l-3.4 2'],
  'tabler:file-text': ['M6 3h8l4 4v14H6z', 'M14 3v5h5', 'M9 13h6', 'M9 17h6'],
  'tabler:heart-rate-monitor': ['M3 12h4l2-4l4 8l2-4h6', 'M12 21a9 9 0 1 1 9-9'],
  'tabler:heartbeat': ['M3 12h4l2-4l4 8l2-4h6'],
  'tabler:language': ['M4 5h9', 'M8 3v2', 'M6 5c0 4 2 7 5 9', 'M4 14c2 0 5-1 7-3', 'M14 7h6', 'M17 5v2', 'M14 19l3-7l3 7', 'M15.3 16h3.4'],
  'tabler:layout-dashboard': ['M4 4h6v6H4z', 'M14 4h6v4h-6z', 'M14 12h6v8h-6z', 'M4 14h6v6H4z'],
  'tabler:layout-grid': ['M4 4h6v6H4z', 'M14 4h6v6h-6z', 'M4 14h6v6H4z', 'M14 14h6v6h-6z'],
  'tabler:mail': ['M4 5h16v14H4z', 'M4 7l8 6l8-6'],
  'tabler:menu-2': ['M4 7h16', 'M4 12h16', 'M4 17h16'],
  'tabler:messages': ['M4 5h16v11H8l-4 3z', 'M8 9h8', 'M8 12h5'],
  'tabler:package': ['M4 7l8-4l8 4v10l-8 4l-8-4z', 'M4 7l8 4l8-4', 'M12 11v10'],
  'tabler:route': ['M6 19a3 3 0 1 0 0-6a3 3 0 0 0 0 6', 'M18 11a3 3 0 1 0 0-6a3 3 0 0 0 0 6', 'M8 15h5a3 3 0 0 0 3-3v-1'],
  'tabler:settings': ['M12 9a3 3 0 1 0 0 6a3 3 0 0 0 0-6', 'M19 12l2-1l-2-3l-2 1a7 7 0 0 0-2-1l-.5-2h-4L10 8a7 7 0 0 0-2 1L6 8l-2 3l2 1v2l-2 1l2 3l2-1a7 7 0 0 0 2 1l.5 2h4l.5-2a7 7 0 0 0 2-1l2 1l2-3l-2-1z'],
  'tabler:share': ['M6 12v7h12v-7', 'M12 3v12', 'M8 7l4-4l4 4'],
  'tabler:shield-lock': ['M12 3l7 3v5c0 5-3 8-7 10c-4-2-7-5-7-10V6z', 'M10 12a2 2 0 1 1 4 0v3h-4z'],
  'tabler:user-circle': ['M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0-18', 'M7 19c1-3 3-4 5-4s4 1 5 4', 'M12 7a3 3 0 1 0 0 6a3 3 0 0 0 0-6'],
}

const paths = computed(() => icons[props.icon] ?? generic)
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    :class="props.class"
    aria-hidden="true"
  >
    <path v-for="path in paths" :key="path" :d="path" />
  </svg>
</template>
