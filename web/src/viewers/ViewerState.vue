<script setup lang="ts">
import { NSpin } from 'naive-ui'
import Dim from '@/components/Dim.vue'

defineProps<{
  loading?: boolean
  error?: string | null
}>()
</script>

<template>
  <div v-if="loading" class="viewer-state">
    <NSpin :size="20" />
  </div>
  <div v-else-if="error" class="viewer-state">
    <Dim>{{ error }}</Dim>
  </div>
  <slot v-else></slot>
</template>

<style lang="scss" scoped>

.viewer-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 160px;
  padding: 24px;
  text-align: center;
}

</style>
