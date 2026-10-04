<script setup lang="ts">
import { NCard, NSpin } from 'naive-ui'
import Row from './Row.vue'

defineProps<{
  loading: boolean
  error: string | null
}>()
</script>

<template>
  <NCard v-if="loading">
    <Row justify="center" gap="12px">
      <NSpin :size="20" />
      <span>Connecting to Rockbreaker host...</span>
    </Row>
  </NCard>
  <NCard v-else-if="error" title="Notice">
    {{ error }}
  </NCard>
  <slot v-else></slot>
</template>
