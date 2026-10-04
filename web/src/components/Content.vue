<script setup lang="ts">
import type { Renderable } from '@/types'

const props = defineProps<{
  content: Renderable
}>()
</script>

<template>
  <template v-if="Array.isArray(content)">
    <Content v-for="part in content" :content="part" />
  </template>
  <template v-else-if="typeof content === 'string'">{{ content }}</template>
  <component v-else :is="content" />
</template>
