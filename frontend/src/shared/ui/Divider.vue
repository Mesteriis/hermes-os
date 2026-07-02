<script setup lang="ts">
import { computed } from 'vue'
import Separator from './Separator.vue'

const props = withDefaults(defineProps<{
	orientation?: 'horizontal' | 'vertical'
	class?: string
}>(), {
	orientation: 'horizontal'
})

const classes = computed(() => props.class)
</script>

<template>
	<Separator :class="classes" :orientation="orientation" />
</template>
